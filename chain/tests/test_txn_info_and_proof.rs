use anyhow::{format_err, Result};
use rand::Rng;
use starcoin_account_api::AccountInfo;
use starcoin_accumulator::Accumulator;
use starcoin_chain_api::{ChainReader, ChainWriter};
use starcoin_config::NodeConfig;
use starcoin_consensus::Consensus;
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::debug;
use starcoin_transaction_builder::{peer_to_peer_txn_sent_as_association, DEFAULT_EXPIRATION_TIME};
use starcoin_types::account_config;
use starcoin_vm_types::access_path::AccessPath;
use starcoin_vm_types::account_address::AccountAddress;
use starcoin_vm_types::account_config::AccountResource;
use starcoin_vm_types::move_resource::MoveResource;
use starcoin_vm_types::state_view::StateReaderExt;
use starcoin_vm_types::transaction::{SignedUserTransaction, Transaction};
use std::collections::HashMap;
use std::sync::Arc;

pub fn gen_txns(seq_num: &mut u64) -> Result<Vec<SignedUserTransaction>> {
    let mut rng = rand::thread_rng();
    let txn_count: u64 = rng.gen_range(1..10);
    let config = Arc::new(NodeConfig::random_for_test());
    debug!("input seq:{}", *seq_num);
    let txns: Vec<SignedUserTransaction> = (0..txn_count)
        .map(|_txn_idx| {
            let account_address = AccountAddress::random();

            let txn = peer_to_peer_txn_sent_as_association(
                account_address,
                *seq_num,
                10000,
                config.net().time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
                config.net(),
            );
            *seq_num += 1;
            txn
        })
        .collect();
    Ok(txns)
}

#[stest::test(timeout = 480)]
fn test_transaction_info_and_proof_1() -> Result<()> {
    let config = Arc::new(NodeConfig::random_for_dag_test());
    let mut block_chain = test_helper::gen_blockchain_for_test(config.net())?;

    let _current_header = block_chain.current_header();
    let miner_account = AccountInfo::random();
    let mut seq_num = 0;
    // generate 5 blocks
    let block_count = 5;
    (0..block_count).for_each(|_| {
        let txns = gen_txns(&mut seq_num).unwrap();
        let (template, _) = block_chain
            .create_block_template(*miner_account.address(), None, txns, vec![], None, None)
            .unwrap();
        let block = block_chain
            .consensus()
            .create_block(template, config.net().time_service().as_ref())
            .unwrap();
        debug!("apply block:{:?}", &block);
        block_chain.apply(block).unwrap();
    });
    let fork_count = 3;
    let fork_point = block_chain
        .get_block_by_number(fork_count)
        .unwrap()
        .unwrap();
    let mut fork_chain = block_chain.fork(fork_point.id()).unwrap();
    let account_reader = fork_chain.chain_state_reader();
    seq_num = account_reader.get_sequence_number(account_config::association_address())?;
    let _txns = gen_txns(&mut seq_num).unwrap();
    let (template, _) = fork_chain
        .create_block_template(
            *miner_account.address(),
            Some(fork_point.header.id()),
            vec![],
            vec![],
            None,
            Some(vec![fork_point.id()]),
        )
        .unwrap();
    let block = fork_chain
        .consensus()
        .create_block(template, config.net().time_service().as_ref())
        .unwrap();
    debug!("Apply block:{:?}", &block);
    fork_chain.apply(block).unwrap();
    assert_eq!(
        block_chain.current_header().id(),
        block_chain.get_block_by_number(5).unwrap().unwrap().id()
    );
    // create latest block
    let account_reader = block_chain.chain_state_reader();
    seq_num = account_reader.get_sequence_number(account_config::association_address())?;
    let _txns = gen_txns(&mut seq_num).unwrap();
    let (template, _) = block_chain
        .create_block_template(*miner_account.address(), None, vec![], vec![], None, None)
        .unwrap();
    let block = block_chain
        .consensus()
        .create_block(template, config.net().time_service().as_ref())
        .unwrap();
    debug!("Apply latest block:{:?}", &block);
    block_chain.apply(block).unwrap();
    assert_eq!(
        block_chain.current_header().id(),
        block_chain.get_block_by_number(6).unwrap().unwrap().id()
    );
    Ok(())
}

#[stest::test(timeout = 480)]
fn test_transaction_info_and_proof() -> Result<()> {
    let config = Arc::new(NodeConfig::random_for_test());
    let mut block_chain = test_helper::gen_blockchain_for_test(config.net())?;
    let mut current_header = block_chain.current_header();
    let miner_account = AccountInfo::random();

    let mut rng = rand::thread_rng();

    let block_count: u64 = rng.gen_range(2..10);
    let mut seq_number = 0;
    let mut all_txns = vec![];
    let mut all_address = HashMap::<HashValue, AccountAddress>::new();

    let genesis_block = block_chain.get_block_by_number(0).unwrap().unwrap();
    //put the genesis txn, the genesis block metadata txn do not generate txn info

    all_txns.push(Transaction::UserTransaction(
        genesis_block.body.transactions.first().cloned().unwrap(),
    ));

    (0..block_count).for_each(|_block_idx| {
        let txn_count: u64 = rng.gen_range(1..10);
        let txns: Vec<SignedUserTransaction> = (0..txn_count)
            .map(|_txn_idx| {
                let account_address = AccountAddress::random();

                let txn = peer_to_peer_txn_sent_as_association(
                    account_address,
                    seq_number,
                    10000,
                    config.net().time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
                    config.net(),
                );
                all_address.insert(txn.id(), account_address);
                seq_number += 1;
                txn
            })
            .collect();

        let (template, _) = block_chain
            .create_block_template(
                *miner_account.address(),
                Some(current_header.id()),
                txns.clone(),
                vec![],
                None,
                None,
            )
            .unwrap();

        let block = block_chain
            .consensus()
            .create_block(template, config.net().time_service().as_ref())
            .unwrap();
        debug!("apply block:{:?}", &block);
        block_chain.apply(block.clone()).unwrap();
        all_txns.push(Transaction::BlockMetadata(
            block.to_metadata(current_header.gas_used()),
        ));
        all_txns.extend(txns.into_iter().map(Transaction::UserTransaction));
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
