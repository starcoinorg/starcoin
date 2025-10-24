use anyhow::Result;
use starcoin_chain::ChainReader;
use starcoin_config::NodeConfig;
use starcoin_logger::prelude::*;
use starcoin_open_block::OpenedBlock;
use starcoin_transaction_builder::DEFAULT_EXPIRATION_TIME;
use starcoin_types::U256;
use starcoin_vm2_crypto::keygen::KeyGen;
use starcoin_vm2_test_helper::{build_transfer_from_association, build_transfer_txn};
use starcoin_vm2_types::{account_address, account_config};
use std::{convert::TryInto, sync::Arc};

#[stest::test]
pub fn test_open_block() -> Result<()> {
    let config = Arc::new(NodeConfig::random_for_test());
    let chain = test_helper::gen_blockchain_for_test(config.net())?;
    let header = chain.current_header();
    let block_gas_limit = 10000000;

    let mut opened_block = {
        // Generate a vm2 AccountAddress for the miner
        let (_private_key, public_key) = KeyGen::from_os_rng().generate_keypair();
        let miner_address = account_address::from_public_key(&public_key);
        OpenedBlock::new(
            chain.get_storage(),
            chain.get_storage2(),
            header.clone(),
            block_gas_limit,
            miner_address, // Use vm2 address
            config.net().time_service().now_millis(),
            vec![],
            U256::from(0),
            chain.consensus(),
            vec![header.id()],      // tips_hash - use current header id for test
            header.version(),       // version from header
            header.pruning_point(), // pruning_point from header
            0,                      // red_blocks - 0 for test
        )?
    };

    let account_reader = chain.chain_state_reader2();
    let association_sequence_num =
        account_reader.get_sequence_number(account_config::association_address())?;
    let (receive_prikey, receive_public_key) = KeyGen::from_os_rng().generate_keypair();
    let receiver = account_address::from_public_key(&receive_public_key);
    let txn1 = build_transfer_from_association(
        receiver,
        association_sequence_num,
        50_000_000,
        config.net().time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
        config.net(),
    )
    .try_into()?;
    let excluded = opened_block.add_transactions(vec![], vec![txn1])?;
    assert_eq!(excluded.discarded_txns.len(), 0);
    assert_eq!(excluded.untouched_txns.len(), 0);

    let template = opened_block.finalize()?;
    let (state_root, state_root1, state_root2) = template.state_roots();
    let parent_info = chain
        .get_block_info(Some(header.id()))?
        .expect("parent block info");
    let parent_vm_state_info = parent_info.get_vm_state_accumulator_info();
    let parent_multi_state = chain.get_storage().get_vm_multi_state(header.id())?;

    assert_eq!(state_root, parent_vm_state_info.accumulator_root);
    assert_eq!(state_root1, parent_multi_state.state_root1());
    assert_eq!(state_root2, parent_multi_state.state_root2());
    assert_eq!(template.gas_used, header.gas_used());
    assert_eq!(template.body.transactions2.len(), 1);

    Ok(())
}
