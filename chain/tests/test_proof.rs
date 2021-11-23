use anyhow::Result;
use consensus::Consensus;
use logger::prelude::debug;
use rand::Rng;
use starcoin_account_api::AccountInfo;
use starcoin_accumulator::Accumulator;
use starcoin_chain_api::{ChainReader, ChainWriter};
use starcoin_config::NodeConfig;
use starcoin_transaction_builder::{peer_to_peer_txn_sent_as_association, DEFAULT_EXPIRATION_TIME};
use starcoin_vm_types::account_address::AccountAddress;
use starcoin_vm_types::transaction::Transaction;
use std::sync::Arc;

#[stest::test(timeout = 480)]
fn test_transaction_info_proof() -> Result<()> {
    let config = Arc::new(NodeConfig::random_for_test());
    let mut block_chain = test_helper::gen_blockchain_for_test(config.net())?;
    let mut parent_header = block_chain.current_header();
    let miner_account = AccountInfo::random();

    let mut rng = rand::thread_rng();

    let block_count: u64 = rng.gen_range(2..10);
    let mut seq_number = 0;
    let mut all_txns = vec![];
    let mut all_address = vec![];
    println!(
        "txn accumulator info: {:?}",
        block_chain.get_txn_accumulator().get_info()
    );
    let genesis_block = block_chain.get_block_by_number(0).unwrap().unwrap();
    //put the genesis txn and genesis block metadata txn
    all_txns.push(Transaction::UserTransaction(
        genesis_block.body.transactions.get(0).cloned().unwrap(),
    ));

    all_txns.push(Transaction::BlockMetadata(genesis_block.to_metadata(0)));

    (0..block_count).for_each(|_block_idx| {
        let txn_count: u64 = rng.gen_range(1..10);
        let txns = (0..txn_count)
            .map(|_txn_idx| {
                let account_address = AccountAddress::random();
                all_address.push(account_address);
                let txn = peer_to_peer_txn_sent_as_association(
                    account_address,
                    seq_number,
                    10000,
                    config.net().time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
                    config.net(),
                );
                seq_number += 1;
                all_txns.push(Transaction::UserTransaction(txn.clone()));
                txn
            })
            .collect();

        let (template, _) = block_chain
            .create_block_template(
                *miner_account.address(),
                Some(parent_header.id()),
                txns,
                vec![],
                None,
            )
            .unwrap();

        let block = block_chain
            .consensus()
            .create_block(template, config.net().time_service().as_ref())
            .unwrap();
        block_chain.apply(block.clone()).unwrap();
        all_txns.push(Transaction::BlockMetadata(
            block.to_metadata(parent_header.gas_used()),
        ));
        parent_header = block.header().clone();
    });

    let txn_index = rng.gen_range(0..all_txns.len());
    debug!("all txns len: {}, txn index:{}", all_txns.len(), txn_index);

    let txn = all_txns.get(txn_index).cloned().unwrap();
    let txn_hash = txn.id();
    let txn_info = block_chain.get_transaction_info(txn_hash)?.unwrap();

    let txn_info_leaf = block_chain
        .get_txn_accumulator()
        .get_leaf(txn_index as u64)?
        .unwrap();
    assert_eq!(txn_info.txn_info.id(), txn_info_leaf);

    let events = block_chain.get_events(txn_info.txn_info.id())?.unwrap();
    let event_index = rng.gen_range(0..events.len()) as u64;
    //let address_index = rng.gen_range(0..all_address.len());
    //let addr = all_address.get(address_index).cloned().unwrap();
    // let access_path =
    //     AccessPath::resource_access_path(association_address(), AccountResource::struct_tag());
    //let txn_block = block_chain.get_block(txn_info.block_id)?.unwrap();

    let txn_proof = block_chain.get_transaction_proof(txn_index as u64, Some(event_index), None)?;

    let result = txn_proof.verify(
        block_chain.current_header().txn_accumulator_root(),
        txn_index as u64,
        Some(event_index),
        None,
    );

    assert!(
        result.is_ok(),
        "{:?} verify failed, reason: {:?}",
        txn_proof,
        result.err().unwrap()
    );

    Ok(())
}
