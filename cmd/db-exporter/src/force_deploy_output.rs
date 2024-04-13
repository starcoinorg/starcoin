// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::fmt::Debug;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::format_err;
use clap::Parser;
use starcoin_chain::{BlockChain, ChainReader, ChainWriter};
use starcoin_cmd::dev::dev_helper;
use starcoin_config::{BuiltinNetworkID, ChainNetwork};
use starcoin_genesis::Genesis;
use starcoin_state_api::ChainStateWriter;
use starcoin_statedb::ChainStateDB;
use starcoin_storage::{
    cache_storage::CacheStorage, db_storage::DBStorage, storage::StorageInstance, Storage,
    StorageVersion,
};
use starcoin_transaction_builder::DEFAULT_MAX_GAS_AMOUNT;
use starcoin_types::account::DEFAULT_EXPIRATION_TIME;
use starcoin_types::{account::Account, block::BlockNumber};
use starcoin_vm_types::{
    access_path::AccessPath,
    account_config::{genesis_address, ModuleUpgradeStrategy, STC_TOKEN_CODE_STR},
    genesis_config::ChainId,
    move_resource::MoveResource,
    state_store::state_key::StateKey,
    state_view::StateView,
    transaction::{RawUserTransaction, Transaction, TransactionPayload},
};

#[derive(Debug, Parser)]
#[clap(
    name = "force-deploy",
    about = "Force deploy output with ignore account upgrade strategy"
)]
pub struct ForceDeployOutput {
    #[clap(long, short = 'n')]
    /// Chain Network, like main, proxima
    pub net: BuiltinNetworkID,

    #[clap(long, short = 'i', parse(from_os_str))]
    /// starcoin node db path. like ~/.starcoin/barnard/starcoindb/db/starcoindb
    pub input_path: PathBuf,

    #[clap(long, short = 'p', parse(from_os_str))]
    /// Package path which
    pub package_path: PathBuf,

    #[clap(long, short = 's')]
    pub block_num: BlockNumber,
}

pub fn force_deploy_output(
    network_path: PathBuf,
    package_path: PathBuf,
    network: BuiltinNetworkID,
    block_number: BlockNumber,
) -> anyhow::Result<()> {
    ::starcoin_logger::init();
    let net = ChainNetwork::new_builtin(network);
    let db_storage = DBStorage::open_with_cfs(
        network_path.join("starcoindb/db/starcoindb"),
        StorageVersion::current_version()
            .get_column_family_names()
            .to_vec(),
        true,
        Default::default(),
        None,
    )?;
    let storage = Arc::new(Storage::new(StorageInstance::new_cache_and_db_instance(
        CacheStorage::new(None),
        db_storage,
    ))?);
    let (chain_info, _) =
        Genesis::init_and_check_storage(&net, storage.clone(), network_path.as_ref())?;
    let chain = BlockChain::new(
        net.time_service(),
        chain_info.head().id(),
        storage.clone(),
        None,
    )
    .expect("create block chain should success.");

    let block = chain
        .get_block_by_number(block_number)?
        .ok_or_else(|| format_err!("{} get block error", block_number))?;

    // BlockChain::set_output_block();
    let mut chain = BlockChain::new(
        net.time_service(),
        block.header.parent_hash(),
        storage,
        None,
    )
    .expect("create block chain should success.");

    // Write upgrade strategy resource to 0
    let upgrade_strategy_path =
        AccessPath::resource_access_path(genesis_address(), ModuleUpgradeStrategy::struct_tag());

    let statedb = chain.chain_state();

    let before_ret = statedb
        .get_state_value(&StateKey::AccessPath(upgrade_strategy_path.clone()))?
        .unwrap();
    assert_eq!(before_ret[0], 1, "Checking the strategy not 1");

    statedb
        .set(&upgrade_strategy_path, vec![0])
        .expect("Add resource failed");

    // Check state is OK
    let after_ret = statedb
        .get_state_value(&StateKey::AccessPath(upgrade_strategy_path))?
        .unwrap();
    assert_eq!(after_ret[0], 0, "Set to upgrade strategy failed!");

    let account = Account::new_association();
    deploy_package(
        network.chain_id(),
        package_path,
        chain.chain_state(),
        &account,
        net.time_service().now_secs(),
    )?;

    Ok(())
}

fn deploy_package(
    chain_id: ChainId,
    package_path: PathBuf,
    statedb: &ChainStateDB,
    account: &Account,
    now_time_by_sec: u64,
) -> anyhow::Result<()> {
    let package = dev_helper::load_package_from_file(&package_path)?;
    let signed_transaction = account.sign_txn(RawUserTransaction::new(
        *account.address(),
        0,
        TransactionPayload::Package(package),
        DEFAULT_MAX_GAS_AMOUNT,
        1,
        now_time_by_sec + DEFAULT_EXPIRATION_TIME,
        chain_id,
        STC_TOKEN_CODE_STR.to_string(),
    ));
    let ret = starcoin_executor::execute_transactions(
        statedb,
        vec![Transaction::UserTransaction(signed_transaction)],
        None,
    )
    .expect("Failed to execute deploy transaction");
    assert_eq!(ret.len(), 1, "There is incorrect execution result");

    Ok(())
}
