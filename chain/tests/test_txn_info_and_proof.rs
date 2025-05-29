use anyhow::{format_err, Result};
use rand::Rng;
use starcoin_account_api::AccountInfo;
use starcoin_accumulator::Accumulator;
use starcoin_chain_api::{ChainReader, ChainWriter};
use starcoin_config::{BuiltinNetworkID, ChainNetwork};
use starcoin_consensus::Consensus;
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::debug;
use starcoin_transaction_builder::{peer_to_peer_txn_sent_as_association, DEFAULT_EXPIRATION_TIME};
use starcoin_types::multi_transaction::MultiSignedUserTransaction;
use starcoin_types::transaction::{StcTransaction, Transaction, Transaction2};
use starcoin_vm_types::access_path::AccessPath;
use starcoin_vm_types::account_address::AccountAddress;
use starcoin_vm_types::account_config::AccountResource;
use starcoin_vm_types::move_resource::MoveResource;
use std::collections::HashMap;

#[stest::test(timeout = 480)]
fn test_transaction_info_and_proof() -> Result<()> {
    let net = ChainNetwork::new_custom(
        "test128".to_string(),
        128.into(),
        BuiltinNetworkID::Test.genesis_config().clone(),
        BuiltinNetworkID::Test.genesis_config2().clone(),
    )
        .unwrap();
    let mut block_chain = test_helper::gen_blockchain_for_test(&net)?;
    let mut current_header = block_chain.current_header();
    let miner_account = AccountInfo::random();

    let mut rng = rand::thread_rng();

    let block_count: u64 = rng.gen_range(2..10);
    let mut seq_number = 0;
    let mut all_txns: Vec<StcTransaction> = vec![];
    let mut all_address = HashMap::<HashValue, AccountAddress>::new();

    let genesis_block = block_chain.get_block_by_number(0).unwrap().unwrap();
    //put the genesis txn, the genesis block metadata txn do not generate txn info

    all_txns.extend_from_slice(&[
        Transaction::UserTransaction(genesis_block.body.transactions.first().cloned().unwrap())
            .into(),
        Transaction2::UserTransaction(genesis_block.body.transactions2.first().cloned().unwrap())
            .into(),
    ]);

    (0..block_count).for_each(|_block_idx| {
        let txn_count: u64 = rng.gen_range(1..10);
        let txns: Vec<MultiSignedUserTransaction> = (0..txn_count)
            .map(|_txn_idx| {
                let account_address = AccountAddress::random();

                let txn = peer_to_peer_txn_sent_as_association(
                    account_address,
                    seq_number,
                    10000,
                    net.time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
                    &net,
                );
                all_address.insert(txn.id(), account_address);
                seq_number += 1;
                txn.into()
            })
            .collect();

        let (template, _) = block_chain
            .create_block_template(
                *miner_account.address(),
                Some(current_header.id()),
                txns.clone(),
                vec![],
                None,
            )
            .unwrap();

        let block = block_chain
            .consensus()
            .create_block(template, net.time_service().as_ref())
            .unwrap();
        block_chain.apply(block.clone()).unwrap();
        all_txns.extend_from_slice(&[Transaction::BlockMetadata(
            block.to_metadata(current_header.gas_used()),
        )
            .into()]);
        all_txns.extend(txns.into_iter().map(|txn| Transaction::from(txn).into()));
        all_txns.extend_from_slice(&[Transaction2::BlockMetadata(
            block.to_metadata2(current_header.gas_used()),
        )
            .into()]);
        current_header = block.header().clone();
    });

    let txn_index = rng.gen_range(0..all_txns.len());
    debug!("all txns len: {}, txn index:{}", all_txns.len(), txn_index);

    for txn_global_index in 0..all_txns.len() {
        let txn = all_txns.get(txn_global_index).cloned().unwrap();
        let txn_hash = txn.id();
        let txn_info = block_chain.get_transaction_info(txn_hash)?.ok_or_else(|| {
            format_err!(
                "Can not get txn info by txn hash:{}, txn:{:?}",
                txn_hash,
                txn
            )
        })?;

        let txn_info_leaf = block_chain
            .get_txn_accumulator()
            .get_leaf(txn_global_index as u64)?
            .unwrap();
        assert_eq!(
            txn_info.transaction_info.id(),
            txn_info_leaf,
            "txn_info hash do not match txn info leaf in accumulator, index: {}",
            txn_global_index
        );

        assert_eq!(
            txn_info.transaction_global_index, txn_global_index as u64,
            "txn_global_index:{}",
            txn_global_index
        );

        let txn = match txn.to_v1() {
            Some(txn) => txn,
            None => {
                continue;
            }
        };

        let account_address = match &txn {
            Transaction::UserTransaction(user_txn) => user_txn.sender(),
            Transaction::BlockMetadata(metadata_txn) => metadata_txn.author(),
        };
        let access_path: Option<AccessPath> = Some(AccessPath::resource_access_path(
            account_address,
            AccountResource::struct_tag(),
        ));

        let events = block_chain
            .get_events(txn_info.transaction_info.id())?
            .unwrap();

        for (event_index, event) in events.into_iter().enumerate() {
            let txn_proof = block_chain
                .get_transaction_proof(
                    current_header.id(),
                    txn_global_index as u64,
                    Some(event_index as u64),
                    access_path.clone(),
                )?
                .expect("get transaction proof return none");
            assert_eq!(&event, &txn_proof.event_proof.as_ref().unwrap().event);

            let result = txn_proof.verify(
                current_header.txn_accumulator_root(),
                txn_global_index as u64,
                Some(event_index as u64),
                access_path.clone(),
            );

            assert!(
                result.is_ok(),
                "txn index: {}, {:?} verify failed, reason: {:?}",
                txn_global_index,
                txn_proof,
                result.err().unwrap()
            );
        }
    }

    Ok(())
}
