use anyhow::format_err;
use starcoin_accumulator::Accumulator;
use starcoin_chain_api::{ChainReader, ChainWriter};
use starcoin_config::NodeConfig;
use starcoin_consensus::Consensus;
use starcoin_statedb::ChainStateDB;
use starcoin_transaction_builder::{build_transfer_from_association, DEFAULT_EXPIRATION_TIME};
use starcoin_types::account_address::AccountAddress;
use starcoin_vm_runtime::force_upgrade_management::get_force_upgrade_block_number;
use starcoin_vm_types::on_chain_config::Version;
use starcoin_vm_types::{account_config, state_view::StateReaderExt};
use std::str::FromStr;
use std::sync::Arc;
use test_helper::executor::get_balance;

#[stest::test]
pub fn test_force_upgrade_1() -> anyhow::Result<()> {
    let config = Arc::new(NodeConfig::random_for_test());

    let force_upgrade_height = get_force_upgrade_block_number(&config.net().chain_id());
    assert!(force_upgrade_height >= 2);
    let initial_blocks = force_upgrade_height - 2;

    let mut miner = test_helper::gen_blockchain_with_blocks_for_test(initial_blocks, config.net())?;
    let block_gas_limit = 10000000;
    let initial_balance = 1000000000000;
    let account_reader = miner.chain_state_reader();
    let association_sequence_num =
        account_reader.get_sequence_number(account_config::association_address())?;
    let miner_db = miner.chain_state();

    let current_version = get_stdlib_version(miner_db)?;
    assert_eq!(current_version, 11);

    // 1 genesis meta + INITIAL_BLOCKS block meta
    let mut txns_num = initial_blocks + 1;
    assert_eq!(miner.get_txn_accumulator().num_leaves(), txns_num);

    // create two txns to deposit some tokens to two black addresses
    // and a third random address which should not in black address list.
    let (black1, txn1, black2, txn2, rand3, txn3) = {
        let receiver1 = AccountAddress::from_str("0xd0c5a06ae6100ce115cad1600fe59e96").unwrap();
        let txn1 = build_transfer_from_association(
            receiver1,
            association_sequence_num,
            initial_balance + 1,
            config.net().time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
            config.net(),
        )
        .try_into()?;

        let receiver2 = AccountAddress::from_str("0x1af80d10cb642adcd9f7fee1420104ec").unwrap();
        let txn2 = build_transfer_from_association(
            receiver2,
            association_sequence_num + 1,
            initial_balance + 2,
            config.net().time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
            config.net(),
        )
        .try_into()?;

        let rand3 = AccountAddress::random();
        let txn3 = build_transfer_from_association(
            rand3,
            association_sequence_num + 2,
            initial_balance + 3,
            config.net().time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
            config.net(),
        )
        .try_into()?;

        (receiver1, txn1, receiver2, txn2, rand3, txn3)
    };

    // block number 1: deposit some stc tokens to two black addresses
    {
        let (block_template, _excluded) = miner
            .create_block_template(
                account_config::association_address(),
                None,
                vec![txn1, txn2, txn3],
                vec![],
                Some(block_gas_limit),
            )
            .unwrap();

        let block = miner
            .consensus()
            .create_block(block_template, miner.time_service().as_ref())?;

        miner.apply(block)?;

        // 1 meta + 3 user = 4 txns
        txns_num += 4;
        assert_eq!(miner.get_txn_accumulator().num_leaves(), txns_num);

        assert_eq!(
            get_balance(black1, miner.chain_state()),
            initial_balance + 1
        );
        assert_eq!(
            get_balance(black2, miner.chain_state()),
            initial_balance + 2
        );
        assert_eq!(get_balance(rand3, miner.chain_state()), initial_balance + 3);
    }

    // fork a new chain, to apply block number 2
    let mut chain_to_apply = miner.fork(miner.current_header().id()).unwrap();

    // create block number 2, then apply it to miner
    let block_num_2 = {
        let (block_template, _excluded) = miner
            .create_block_template(
                account_config::association_address(),
                None,
                vec![],
                vec![],
                Some(block_gas_limit),
            )
            .unwrap();

        let block2 = miner
            .consensus()
            .create_block(block_template, miner.time_service().as_ref())?;

        miner.apply(block2.clone())?;

        // 1 meta + 1 extra = 2 txns
        let txns_num = txns_num + 2;
        assert_eq!(miner.get_txn_accumulator().num_leaves(), txns_num);

        assert_eq!(
            get_balance(black1, miner.chain_state()),
            0,
            "Upgrade Failed, Balance of black list account not 0"
        );

        assert_eq!(
            get_balance(black2, miner.chain_state()),
            0,
            "Upgrade Failed, Balance of black list account not 0"
        );

        assert_eq!(get_balance(rand3, miner.chain_state()), initial_balance + 3);

        block2
    };

    // apply block number 2 to another chain
    {
        // !!!non-zero balance
        assert_ne!(get_balance(black1, chain_to_apply.chain_state()), 0);
        assert_ne!(get_balance(black2, chain_to_apply.chain_state()), 0);
        assert_ne!(get_balance(rand3, chain_to_apply.chain_state()), 0);

        chain_to_apply.apply(block_num_2)?;

        // 1 meta + 1 extra = 2 txns
        let txns_num = txns_num + 2;
        assert_eq!(chain_to_apply.get_txn_accumulator().num_leaves(), txns_num);

        assert_eq!(get_balance(black1, chain_to_apply.chain_state()), 0);
        assert_eq!(get_balance(black2, chain_to_apply.chain_state()), 0);
        assert_eq!(
            get_balance(rand3, chain_to_apply.chain_state()),
            initial_balance + 3
        );
    }

    // Check on chain config for v12
    let upgraded_version = get_stdlib_version(chain_to_apply.chain_state())?;
    assert_eq!(upgraded_version, 12);

    Ok(())
}

#[stest::test]
fn test_force_upgrade_2() -> anyhow::Result<()> {
    let config = Arc::new(NodeConfig::random_for_test());

    let force_upgrade_height = get_force_upgrade_block_number(&config.net().chain_id());
    assert!(force_upgrade_height >= 2);

    let chain =
        test_helper::gen_blockchain_with_blocks_for_test(force_upgrade_height, config.net())?;

    // genesis 1 + 1meta in each blocks  + special block 1meta+1extra.txn
    assert_eq!(
        chain.get_txn_accumulator().num_leaves(),
        force_upgrade_height + 2
    );

    let chain =
        test_helper::gen_blockchain_with_blocks_for_test(force_upgrade_height + 1, config.net())?;
    // genesis 1 + 1meta in each blocks + special block 2 + 1 meta in last block
    assert_eq!(
        chain.get_txn_accumulator().num_leaves(),
        force_upgrade_height + 3
    );

    Ok(())
}

fn get_stdlib_version(chain_state_db: &ChainStateDB) -> anyhow::Result<u64> {
    chain_state_db
        .get_on_chain_config::<Version>()?
        .map(|version| version.major)
        .ok_or_else(|| format_err!("on chain config stdlib version can not be empty."))
}
