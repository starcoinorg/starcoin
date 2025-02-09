use std::str::FromStr;
use std::sync::Arc;

use anyhow::format_err;

use starcoin_accumulator::Accumulator;
use starcoin_chain::BlockChain;
use starcoin_chain_api::{ChainReader, ChainWriter};
use starcoin_config::NodeConfig;
use starcoin_consensus::Consensus;
use starcoin_logger::prelude::info;
use starcoin_statedb::ChainStateDB;
use starcoin_transaction_builder::{
    build_transfer_from_association, frozen_config_do_burn_frozen_from_association,
    frozen_config_update_burn_block_number_by_association, DEFAULT_EXPIRATION_TIME,
};
use starcoin_types::account_address::AccountAddress;
use starcoin_types::account_config::CORE_CODE_ADDRESS;
use starcoin_types::block::Block;
use starcoin_types::identifier::Identifier;
use starcoin_types::language_storage::StructTag;
use starcoin_vm_runtime::force_upgrade_management::get_force_upgrade_block_number;
use starcoin_vm_types::access_path::AccessPath;
use starcoin_vm_types::account_config::{
    association_address, genesis_address, FrozenConfigBurnBlockNumberResource,
};
use starcoin_vm_types::on_chain_config::Version;
use starcoin_vm_types::on_chain_resource::Treasury;
use starcoin_vm_types::transaction::SignedUserTransaction;
use starcoin_vm_types::{account_config, state_view::StateReaderExt};
use test_helper::executor::{execute_and_apply, get_balance};

#[stest::test]
pub fn test_force_upgrade_1() -> anyhow::Result<()> {
    let config = Arc::new(NodeConfig::random_for_test());
    let net = config.net();

    let force_upgrade_height = get_force_upgrade_block_number(&net.chain_id());
    assert!(force_upgrade_height >= 2);
    let initial_blocks = force_upgrade_height - 2;

    let mut miner = test_helper::gen_blockchain_with_blocks_for_test(initial_blocks, net)?;
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
            net.time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
            net,
        )
        .try_into()?;

        let receiver2 = AccountAddress::from_str("0x1af80d10cb642adcd9f7fee1420104ec").unwrap();
        let txn2 = build_transfer_from_association(
            receiver2,
            association_sequence_num + 1,
            initial_balance + 2,
            net.time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
            net,
        )
        .try_into()?;

        let rand3 = AccountAddress::random();
        let txn3 = build_transfer_from_association(
            rand3,
            association_sequence_num + 2,
            initial_balance + 3,
            net.time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
            net,
        )
        .try_into()?;

        (receiver1, txn1, receiver2, txn2, rand3, txn3)
    };

    // block number 1: deposit some stc tokens to two black addresses
    {
        execute_transactions_by_miner(&mut miner, vec![txn1, txn2, txn3])?;

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

    let forked_txn_num = txns_num;

    // fork a new chain, to apply block number 2
    let mut chain_to_apply = miner.fork(miner.current_header().id()).unwrap();

    // create block number 2, then apply it to miner
    let block_num_2 = {
        let block2 = execute_transactions_by_miner(&mut miner, vec![])?;

        // 1 meta + 1 extra = 2 txns
        txns_num += 2;
        assert_eq!(miner.get_txn_accumulator().num_leaves(), txns_num);

        let black1_balance = get_balance(black1, miner.chain_state());
        info!("Black 1 balance is: {:?}", black1_balance);
        assert_eq!(
            black1_balance,
            initial_balance + 1,
            "Force-Upgrading Failed, Balance of black-1 account changed!"
        );

        let black2_balance = get_balance(black2, miner.chain_state());
        info!("Black 2 balance is: {:?}", black2_balance);
        assert_eq!(
            black2_balance,
            initial_balance + 2,
            "Force-upgrading Failed, Balance of black-2 account changed!"
        );

        assert_eq!(get_balance(rand3, miner.chain_state()), initial_balance + 3);

        block2
    };

    // Upgrade script will create a new signer on the behalf of the association account,
    // the sequence number of the association account will be increased by 1.
    {
        assert_eq!(
            miner
                .chain_state_reader()
                .get_sequence_number(account_config::association_address())
                .unwrap(),
            association_sequence_num + 4
        );
    }

    // Apply block number 3, this will call FrozenConfigStrategy::do_burn_frozen
    let block_num_3 = {
        let block3 = gen_empty_block_for_miner(&mut miner)?;
        info!(
            "After gen_empty_block_for_miner, current block_number {:?}",
            miner.status().head().number()
        );

        // Check force upgrade from miner
        assert_eq!(get_stdlib_version(miner.chain_state())?, 12);

        let current_block_num = { miner.status().head().number() };
        let expect_burn_block_number = current_block_num + 2; // 53

        // Set burn block number into chain
        let _block4 = execute_transactions_by_miner(
            &mut miner,
            vec![frozen_config_update_burn_block_number_by_association(
                association_sequence_num + 4,
                net,
                expect_burn_block_number,
            )?],
        )?;

        let read_burn_block_number = miner
            .chain_state()
            .get_resource::<FrozenConfigBurnBlockNumberResource>(association_address())?
            .unwrap();
        assert_eq!(read_burn_block_number.block_number(), expect_burn_block_number);

        // Check not equal 0
        assert_ne!(
            miner.chain_state_reader().get_balance(black1)?.unwrap(),
            0,
            "Burning Failed, Balance of black-1 account is not 0"
        );
        assert_ne!(
            miner.chain_state_reader().get_balance(black2)?.unwrap(),
            0,
            "Burning Failed, Balance of black-2 account is not 0"
        );

        // Block number: 52, Check abort txn_status: Keep(ABORTED { code: 27137
        let _block5 = execute_transactions_by_miner(
            &mut miner,
            vec![frozen_config_do_burn_frozen_from_association(
                association_sequence_num + 5,
                net,
            )?],
        )?;

        // Block 53
        let _block6 = gen_empty_block_for_miner(&mut miner);


        // Block number: 54, Execute Succeed
        let _block7 = execute_transactions_by_miner(
            &mut miner,
            vec![frozen_config_do_burn_frozen_from_association(
                association_sequence_num + 7,
                net,
            )?],
        );

        // Check eq 0
        assert_eq!(
            miner.chain_state_reader().get_balance(black1)?.unwrap(),
            0,
            "Burning Failed, Balance of black-1 account is not 0"
        );
        assert_eq!(
            miner.chain_state_reader().get_balance(black2)?.unwrap(),
            0,
            "Burning Failed, Balance of black-2 account is not 0"
        );

        // 1 meta + 2 txns = 3 txns
        // txns_num += 2;
        // let leaves_num = miner.get_txn_accumulator().num_leaves();
        // assert_eq!(leaves_num, txns_num);

        // let black1_balance = get_balance(black1, miner.chain_state());
        // println!("Black 1 balance is: {:?}", black1_balance);
        // assert_eq!(
        //     black1_balance, 0,
        //     "Burning Failed, Balance of black-1 account is not 0"
        // );
        //
        // let black2_balance = get_balance(black2, miner.chain_state());
        // println!("Black 2 balance is: {:?}", black2_balance);
        // assert_eq!(
        //     black2_balance, 0,
        //     "Burning Failed, Balance of black-2 account is not 0"
        // );
        // block3
    };

    // apply block number 2,3 to another chain
    // {
    //     // !!!non-zero balance
    //     assert_ne!(get_balance(black1, chain_to_apply.chain_state()), 0);
    //     assert_ne!(get_balance(black2, chain_to_apply.chain_state()), 0);
    //     assert_ne!(get_balance(rand3, chain_to_apply.chain_state()), 0);
    //
    //     chain_to_apply.apply(block_num_2)?;
    //
    //     // 1 meta + 1 extra = 2 txns
    //     let txns_num = forked_txn_num + 2;
    //     assert_eq!(chain_to_apply.get_txn_accumulator().num_leaves(), txns_num);
    //
    //     assert_eq!(
    //         get_balance(black1, chain_to_apply.chain_state()),
    //         initial_balance + 1
    //     );
    //     assert_eq!(
    //         get_balance(black2, chain_to_apply.chain_state()),
    //         initial_balance + 2
    //     );
    //     assert_eq!(
    //         get_balance(rand3, chain_to_apply.chain_state()),
    //         initial_balance + 3
    //     );
    //
    //     chain_to_apply.apply(block_num_3)?;
    //
    //     // 1 meta + 2 txns = 3 txns
    //     let txns_num = txns_num + 3;
    //     let leaves_num = miner.get_txn_accumulator().num_leaves();
    //     assert_eq!(leaves_num, txns_num);
    //
    //     assert_eq!(get_balance(black1, chain_to_apply.chain_state()), 0);
    //     assert_eq!(get_balance(black2, chain_to_apply.chain_state()), 0);
    //     assert_eq!(
    //         get_balance(rand3, chain_to_apply.chain_state()),
    //         initial_balance + 3
    //     );
    // }
    //
    // // Check on chain config for v12
    // let upgraded_version = get_stdlib_version(chain_to_apply.chain_state())?;
    // assert_eq!(upgraded_version, 12);

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

fn gen_empty_block_for_miner(miner: &mut BlockChain) -> anyhow::Result<Block> {
    execute_transactions_by_miner(miner, vec![])
}

fn execute_transactions_by_miner(
    miner: &mut BlockChain,
    user_txns: Vec<SignedUserTransaction>,
) -> anyhow::Result<Block> {
    let (block_template, _excluded) = miner.create_block_template(
        account_config::association_address(),
        None,
        user_txns,
        vec![],
        Some(10000000),
    )?;

    let block = miner
        .consensus()
        .create_block(block_template, miner.time_service().as_ref())?;

    miner.apply(block.clone())?;

    Ok(block)
}
