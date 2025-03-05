use std::str::FromStr;
use std::sync::Arc;

use anyhow::format_err;

use starcoin_accumulator::Accumulator;
use starcoin_chain::BlockChain;
use starcoin_chain_api::{ChainReader, ChainWriter};
use starcoin_config::{ChainNetwork, NodeConfig};
use starcoin_consensus::Consensus;
use starcoin_logger::prelude::info;
use starcoin_statedb::ChainStateDB;
use starcoin_transaction_builder::{
    build_transfer_from_association, build_transfer_txn,
    frozen_config_do_burn_frozen_from_association,
    frozen_config_update_burn_block_number_by_association, peer_to_peer_txn_sent_as_association,
    peer_to_peer_v2, DEFAULT_EXPIRATION_TIME,
};
use starcoin_types::account::Account;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::block::Block;
use starcoin_types::vm_error::StatusCode;
use starcoin_vm_runtime::force_upgrade_management::{
    get_force_upgrade_account, get_force_upgrade_block_number,
};
use starcoin_vm_types::{
    account_config::{
        self, association_address, core_code_address, FrozenConfigBurnBlockNumberResource,
    },
    identifier::Identifier,
    language_storage::ModuleId,
    on_chain_config::Version,
    state_view::StateReaderExt,
    transaction::{RawUserTransaction, ScriptFunction, SignedUserTransaction, TransactionPayload},
};
use test_helper::executor::{get_balance, get_sequence_number};

#[stest::test]
pub fn test_force_upgrade_1() -> anyhow::Result<()> {
    let config = Arc::new(NodeConfig::random_for_test());
    let net = config.net();

    let force_upgrade_height = get_force_upgrade_block_number(&net.chain_id());
    assert!(force_upgrade_height >= 2);
    let initial_blocks = force_upgrade_height - 2;

    let mut miner = test_helper::gen_blockchain_with_blocks_for_test(initial_blocks, net)?;
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

    let _forked_txn_num = txns_num;

    // create block number 2, then apply it to miner
    //let _block_num_2 = {
    {
        let account = get_force_upgrade_account(&config.net().chain_id()).unwrap();

        let balance1 = get_balance(*account.address(), miner.chain_state());
        let block2 = execute_transactions_by_miner(&mut miner, vec![])?;
        let balance2 = get_balance(*account.address(), miner.chain_state());

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

        assert_eq!(balance1, balance2);

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
    // let _block_num_3 = {
    {
        let _block3 = gen_empty_block_for_miner(&mut miner)?;
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
        assert_eq!(
            read_burn_block_number.block_number(),
            expect_burn_block_number
        );

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

        // Block nubmer: 53, generate empty block
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
    };

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

#[stest::test]
fn test_frozen_account() -> anyhow::Result<()> {
    let config = Arc::new(NodeConfig::random_for_test());

    let force_upgrade_height = get_force_upgrade_block_number(&config.net().chain_id());
    assert!(force_upgrade_height >= 2);

    let mut chain =
        test_helper::gen_blockchain_with_blocks_for_test(force_upgrade_height + 1, config.net())?;

    let net = config.net();
    let association_sequence_num = chain
        .chain_state_reader()
        .get_sequence_number(association_address())?;
    let black = AccountAddress::from_str("0xd0c5a06ae6100ce115cad1600fe59e96").unwrap();

    // It's ok to send txn to black address
    {
        let black_as_receiver_txn = peer_to_peer_txn_sent_as_association(
            black,
            association_sequence_num,
            1,
            net.time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
            net,
        );

        assert!(starcoin_executor::validate_transaction(
            chain.chain_state(),
            black_as_receiver_txn,
            None
        )
        .is_none());
    }

    // It's not ok to use a black address as sender
    {
        let black_as_sender_txn = net
            .genesis_config()
            .sign_with_association(build_transfer_txn(
                black,
                association_address(),
                association_sequence_num,
                1,
                1,
                1_000_000,
                net.time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
                net.chain_id(),
            ))
            .unwrap();

        assert_eq!(
            starcoin_executor::validate_transaction(chain.chain_state(), black_as_sender_txn, None)
                .unwrap()
                .status_code(),
            StatusCode::SENDING_ACCOUNT_FROZEN
        );
    }

    Ok(())
}

#[stest::test]
fn test_frozen_for_global_frozen() -> anyhow::Result<()> {
    let config = Arc::new(NodeConfig::random_for_test());

    let force_upgrade_height = get_force_upgrade_block_number(&config.net().chain_id());
    assert!(force_upgrade_height >= 2);

    let mut chain =
        test_helper::gen_blockchain_with_blocks_for_test(force_upgrade_height + 1, config.net())?;

    let net = config.net();
    let random_user_account = Account::new();
    let amount = 1000000000;

    let random_user_seq_num =
        get_sequence_number(*random_user_account.address(), chain.chain_state());
    let mut association_seq_num = get_sequence_number(association_address(), chain.chain_state());

    // It's ok to send txn to black address
    {
        // Send STC to black user
        execute_transactions_by_miner(
            &mut chain,
            vec![peer_to_peer_txn_sent_as_association(
                *random_user_account.address(),
                association_seq_num,
                amount,
                net.time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
                net,
            )],
        )?;
        assert_eq!(
            chain
                .chain_state_reader()
                .get_balance(*random_user_account.address())?
                .unwrap(),
            amount,
            "Failed to get balance"
        );
    }

    // It's not ok by switch global frozen open
    {
        association_seq_num += 1;
        execute_transactions_by_miner(
            &mut chain,
            vec![build_global_frozen_txn_sign_with_association(
                true,
                association_seq_num,
                net,
            )?],
        )?;

        let transfer_to_association_txn = peer_to_peer_v2(
            &random_user_account,
            &association_address(),
            random_user_seq_num,
            amount,
            net,
        );

        assert_eq!(
            starcoin_executor::validate_transaction(
                chain.chain_state(),
                transfer_to_association_txn,
                None
            )
            .unwrap()
            .status_code(),
            StatusCode::SEND_TXN_GLOBAL_FROZEN
        );
    }

    // It's ok by switch global frozen closed
    {
        association_seq_num += 1;
        execute_transactions_by_miner(
            &mut chain,
            vec![build_global_frozen_txn_sign_with_association(
                false,
                association_seq_num,
                net,
            )?],
        )?;

        let transfer_to_association_txn = peer_to_peer_v2(
            &random_user_account,
            &association_address(),
            random_user_seq_num,
            amount,
            net,
        );

        assert!(starcoin_executor::validate_transaction(
            chain.chain_state(),
            transfer_to_association_txn,
            None
        )
        .is_none());
    }

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
    let miner_account = Account::new();
    let (block_template, _excluded) = miner.create_block_template(
        *miner_account.address(),
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

pub fn build_global_frozen_txn_sign_with_association(
    frozen: bool,
    seq_num: u64,
    net: &ChainNetwork,
) -> anyhow::Result<SignedUserTransaction> {
    net.genesis_config()
        .sign_with_association(RawUserTransaction::new_with_default_gas_token(
            association_address(),
            seq_num,
            TransactionPayload::ScriptFunction(ScriptFunction::new(
                ModuleId::new(
                    core_code_address(),
                    Identifier::new("FrozenConfigStrategy").unwrap(),
                ),
                Identifier::new("set_global_frozen").unwrap(),
                vec![],
                vec![bcs_ext::to_bytes(&frozen).unwrap()],
            )),
            1_000_000,
            1,
            net.time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
            net.chain_id(),
        ))
}
