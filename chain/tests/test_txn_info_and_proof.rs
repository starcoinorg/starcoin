use anyhow::{format_err, Result};
use rand::Rng;
use starcoin_account_api::AccountInfo;
use starcoin_accumulator::node::AccumulatorStoreType;
use starcoin_accumulator::{Accumulator, MerkleAccumulator};
use starcoin_chain_api::{ChainReader, ChainWriter};
use starcoin_config::upgrade_config::vm1_offline_height;
use starcoin_config::{BuiltinNetworkID, ChainNetwork};
use starcoin_consensus::Consensus;
use starcoin_crypto::HashValue;
use starcoin_transaction_builder::{peer_to_peer_txn_sent_as_association, DEFAULT_EXPIRATION_TIME};
use starcoin_types::block_metadata;
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

    let mut rng = rand::rng();

    let block_count: u64 = rng.random_range(2..10);
    let mut seq_number = 0;
    let mut all_txns: Vec<StcTransaction> = vec![];
    let mut executed_blocks = vec![];
    let mut all_address = HashMap::<HashValue, AccountAddress>::new();

    let genesis_block = block_chain.get_block_by_number(0).unwrap().unwrap();
    let vm1_offline_number =
        vm1_offline_height(block_chain.current_header().chain_id().id().into());

    //put the genesis txn, the genesis block metadata txn do not generate txn info

    all_txns.extend_from_slice(&[
        Transaction2::UserTransaction(genesis_block.body.transactions2.first().cloned().unwrap())
            .into(),
        Transaction::UserTransaction(genesis_block.body.transactions.first().cloned().unwrap())
            .into(),
    ]);

    executed_blocks.push(genesis_block);
    let execution_result: Result<()> = (0..block_count).try_for_each(|_block_idx| {
        let txn_count: u64 = rng.random_range(1..10);
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
            .create_block_template_simple_with_txns(*miner_account.address(), txns.clone())
            .unwrap();

        let block = block_chain
            .consensus()
            .create_block(template, net.time_service().as_ref())
            .unwrap();
        let executed_block = block_chain.apply(block.clone())?;
        all_txns.extend_from_slice(&[Transaction2::BlockMetadata(
            block.to_metadata(current_header.gas_used(), 0),
        )
        .into()]);

        executed_blocks.push(executed_block.block().clone());

        if vm1_offline_number > block.header().number() {
            all_txns.extend_from_slice(&[Transaction::BlockMetadata(block_metadata::from(
                block.to_metadata(current_header.gas_used(), 0),
            ))
            .into()]);
            all_txns.extend(txns.into_iter().map(|txn| Transaction::from(txn).into()));
        }
        current_header = block.header().clone();

        Ok(())
    });
    assert!(
        execution_result.is_ok(),
        "execute block error: {:#?}",
        execution_result
    );

    let storage1 = block_chain.get_storage();
    let _storage2 = block_chain.get_storage2();

    let current_block_info = storage1
        .get_block_info(current_header.id())?
        .ok_or_else(|| format_err!("current block info is not found"))?;
    let final_transaction_accumulator = MerkleAccumulator::new_with_info(
        current_block_info.txn_accumulator_info,
        storage1.get_accumulator_store(AccumulatorStoreType::Transaction),
    );
    let final_transaction_info_index = final_transaction_accumulator
        .num_leaves()
        .checked_sub(1)
        .ok_or_else(|| format_err!("proof transaction leaf number is overflow"))?;
    let final_transaction_info_id = final_transaction_accumulator
        .get_leaf(final_transaction_info_index)?
        .ok_or_else(|| format_err!("final transaction info is not found"))?;

    let final_transaction_info = storage1
        .get_transaction_info(final_transaction_info_id)?
        .ok_or_else(|| format_err!("final transaction info is not found"))?
        .transaction_info
        .to_v1()
        .ok_or_else(|| format_err!("final transaction info is not found"))?;
    let final_state_root_hash = final_transaction_info
        .state_root_hash()
        .ok_or_else(|| format_err!("final transaction info's state root is not found"))?;
    let final_raw_transaction_id = final_transaction_info.transaction_hash();
    let final_raw_transaction = storage1
        .get_transaction(final_raw_transaction_id)?
        .ok_or_else(|| format_err!("Cannot find txn by hash:{}", final_raw_transaction_id,))?
        .to_v1()
        .ok_or_else(|| format_err!("final transaction info is not found"))?;
    let final_account_address = match &final_raw_transaction {
        Transaction::UserTransaction(user_txn) => user_txn.sender(),
        Transaction::BlockMetadata(metadata_txn) => metadata_txn.author(),
    };
    let final_access_path: Option<AccessPath> = Some(AccessPath::resource_access_path(
        final_account_address,
        AccountResource::struct_tag(),
    ));

    let mut transaction_accumulator_index_begin: u64 = 0;
    for block in executed_blocks {
        let mut transactions: Vec<StcTransaction> = vec![];

        let (_state_root_index1, _state_root_index2) = if block.header().is_genesis() {
            transactions.extend(vec![
                Transaction2::UserTransaction(block.body.transactions2.first().cloned().unwrap())
                    .into(),
                Transaction::UserTransaction(block.body.transactions.first().cloned().unwrap())
                    .into(),
            ]);
            (1, 0)
        } else {
            let parent = block_chain
                .get_block(block.header().parent_hash())?
                .ok_or_else(|| {
                    format_err!("Cannot get block by hash: {}", block.header().parent_hash())
                })?;
            transactions.push(
                Transaction2::BlockMetadata(block.to_metadata(parent.header().gas_used(), 0))
                    .into(),
            );
            let state_root_index1 = if vm1_offline_number > block.header().number() {
                transactions.push(
                    Transaction::BlockMetadata(block_metadata::from(
                        block.to_metadata(parent.header().gas_used(), 0),
                    ))
                    .into(),
                );
                transactions.extend(block.body.transactions.iter().map(|txn| {
                    Transaction::from(MultiSignedUserTransaction::VM1(txn.clone())).into()
                }));
                transactions.len().saturating_sub(1)
            } else {
                usize::MAX
            };
            transactions.extend(
                block.body.transactions2.iter().map(|txn| {
                    Transaction::from(MultiSignedUserTransaction::VM2(txn.clone())).into()
                }),
            );
            let state_root_index2 = transactions.len().saturating_sub(1);
            (state_root_index1, state_root_index2)
        };

        let transaction_count = transactions.len() as u64;
        for (index, txn) in transactions.into_iter().enumerate() {
            let txn_global_index = transaction_accumulator_index_begin.saturating_add(index as u64);
            let txn_info_leaf = block_chain
                .get_txn_accumulator()
                .get_leaf(txn_global_index)?
                .ok_or_else(|| format_err!("Cannot get txn info by index: {}", txn_global_index))?;
            let txn_info = block_chain.get_transaction_info(txn.id())?.ok_or_else(|| {
                format_err!(
                    "Cannot get txn info by txn hash:{}, index: {}",
                    txn.id(),
                    index
                )
            })?;
            assert_eq!(
                txn_info.transaction_global_index, txn_global_index,
                "txn info global index do not match, txn info index: {}, txn_global_index:{}",
                txn_info.transaction_global_index, txn_global_index
            );
            assert_eq!(
                txn_info.transaction_info.id(),
                txn_info_leaf,
                "txn_info hash do not match txn info leaf in accumulator, index: {}",
                txn_global_index
            );

            let txn = match txn.to_v1() {
                Some(txn) => txn,
                None => {
                    continue;
                }
            };

            // if index == state_root_index1 || index == 1 {
            // 1 is the block metadata txn of vm1
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
                        txn_global_index,
                        Some(event_index as u64),
                        access_path.clone(),
                    )?
                    .expect("get transaction proof return none");
                assert_eq!(&event, &txn_proof.event_proof.as_ref().unwrap().event);

                let result = txn_proof.verify(
                    current_header.txn_accumulator_root(),
                    txn_global_index,
                    final_transaction_info_index,
                    final_transaction_info_id,
                    Some(event_index as u64),
                    access_path.clone(),
                    final_access_path.clone(),
                    final_state_root_hash,
                );

                assert!(
                    result.is_ok(),
                    "txn index: {}, {:?} verify failed, reason: {:?}",
                    txn_global_index,
                    txn_proof,
                    result.err().unwrap()
                );
            }
            // } else {
            //     let info = txn_info.transaction_info.to_v1().ok_or_else(|| {
            //         format_err!(
            //             "Cannot get txn info by txn hash:{}, index: {}",
            //             txn.id(),
            //             index
            //         )
            //     })?;
            //     assert!(
            //         info.state_root_hash().is_none(),
            //         "state root hash should be none, index: {}, state root index1: {}",
            //         index,
            //         state_root_index1
            //     );
            // }
        }
        transaction_accumulator_index_begin =
            transaction_accumulator_index_begin.saturating_add(transaction_count);
    }

    Ok(())
}
