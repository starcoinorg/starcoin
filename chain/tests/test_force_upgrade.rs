use starcoin_account_api::AccountInfo;
use starcoin_chain_api::ChainReader;
use starcoin_config::NodeConfig;
use starcoin_crypto::keygen::KeyGen;
use starcoin_open_block::OpenedBlock;
use starcoin_transaction_builder::{build_transfer_from_association, DEFAULT_EXPIRATION_TIME};
use starcoin_types::{account_address, U256};
use starcoin_vm_runtime::force_upgrade_management;
use starcoin_vm_types::{account_config, state_view::StateReaderExt};
use std::sync::Arc;
use anyhow::format_err;
use starcoin_state_api::ChainStateWriter;
use starcoin_statedb::ChainStateDB;
use starcoin_types::account::{AccountData};
use starcoin_vm_types::account_config::{STC_TOKEN_CODE_STR};
use starcoin_vm_types::on_chain_config::Version;

#[stest::test]
pub fn test_force_upgrade_in_openblock() -> anyhow::Result<()> {
    let config = Arc::new(NodeConfig::random_for_test());
    let chain = test_helper::gen_blockchain_for_test(config.net())?;
    let header = chain.current_header().clone();

    let block_gas_limit = 10000000;

    let account_reader = chain.chain_state_reader();
    let association_sequence_num =
        account_reader.get_sequence_number(account_config::association_address())?;

    let mut opened_block = {
        let miner_account = AccountInfo::random();
        OpenedBlock::new(
            chain.get_storage(),
            header,
            block_gas_limit,
            miner_account.address,
            config.net().time_service().now_millis(),
            vec![],
            U256::from(0),
            chain.consensus(),
            None,
        )?
    };

    let statedb = chain.get_chain_state_db();

    let inited_balance = 1000000000000;

    /// Add stc to black accounts from black list v1
    let black_user_1 = AccountData::with_account(
        force_upgrade_management::create_account(
            "7e8a25de99416dd5a96fb2a804da7f2f93ff0ece42bfe91572bd2312be812ce5",
        )?,
        inited_balance,
        STC_TOKEN_CODE_STR,
        0,
    );
    let black_user_2 = AccountData::with_account(
        force_upgrade_management::create_account(
            "005520f06177cd358bd2de4c6783eeb9608216d1fda9e91e50020a4ac261afed",
        )?,
        inited_balance,
        STC_TOKEN_CODE_STR,
        0,
    );
    statedb.apply_write_set(black_user_1.to_writeset());
    statedb.apply_write_set(black_user_2.to_writeset());

    // let before_version =  get_stdlib_version(statedb)?;
    // assert_eq!(before_version, 11, "Upgrade failed, got wrong number!");

    let (_receive_prikey, receive_public_key) = KeyGen::from_os_rng().generate_keypair();
    let receiver = account_address::from_public_key(&receive_public_key);
    let txn1 = build_transfer_from_association(
        receiver,
        association_sequence_num,
        50_000_000,
        config.net().time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
        config.net(),
    )
    .try_into()?;

    let excluded = opened_block.push_txns(vec![txn1])?;
    assert_eq!(excluded.discarded_txns.len(), 0);
    assert_eq!(excluded.untouched_txns.len(), 0);

    opened_block.finalize()?;

    // Check on chain config for v12
    // assert_eq!(get_stdlib_version(statedb)?, 12, "Upgrade failed, got wrong number!");


    // Check black list balance
    let balance_1 = chain.get_chain_state_db().get_balance(black_user_1.address().clone())?.unwrap();
    assert_eq!(balance_1, 0, "Upgrade Faild, Balance of black list account not 0");

    let balance_2 = chain.get_chain_state_db().get_balance(black_user_2.address().clone())?.unwrap();
    assert_eq!(balance_2, 0, "Upgrade Faild, Balance of black list account not 0");

    // execute_transactions(statedb, );

    Ok(())
}

fn get_stdlib_version(chain_state_db: &ChainStateDB) -> anyhow::Result<u64> {
    chain_state_db
        .get_on_chain_config::<Version>()?
        .map(|version| version.major)
        .ok_or_else(|| format_err!("on chain config stdlib version can not be empty."))
}