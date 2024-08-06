use anyhow::Result;
use rand::Rng;
use starcoin_account_api::AccountInfo;
use starcoin_chain_api::{ChainReader, ChainWriter};
use starcoin_config::NodeConfig;
use starcoin_consensus::Consensus;
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::debug;
use starcoin_transaction_builder::{peer_to_peer_txn_sent_as_association, DEFAULT_EXPIRATION_TIME};
use starcoin_types::account_config;
use starcoin_vm_types::account_address::AccountAddress;
use starcoin_vm_types::state_view::StateReaderExt;
use starcoin_vm_types::transaction::SignedUserTransaction;
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
            .create_block_template(
                *miner_account.address(),
                None,
                txns,
                vec![],
                None,
                vec![],
                HashValue::zero(),
            )
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
            vec![fork_point.id()],
            HashValue::zero(),
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
        .create_block_template(
            *miner_account.address(),
            None,
            vec![],
            vec![],
            None,
            vec![],
            HashValue::zero(),
        )
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
