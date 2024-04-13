// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::fmt::Debug;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::format_err;
use clap::Parser;
use starcoin_chain::BlockChain;
use starcoin_chain_api::FORCE_UPGRADE_PACKAGE;
use starcoin_config::{BuiltinNetworkID, ChainNetwork, RocksdbConfig};
use starcoin_crypto::ed25519::{Ed25519PrivateKey, Ed25519PublicKey};
use starcoin_genesis::Genesis;
use starcoin_statedb::{ChainStateDB, ChainStateWriter};
use starcoin_storage::{
    cache_storage::CacheStorage, db_storage::DBStorage, storage::StorageInstance, Storage,
};
use starcoin_types::account::{Account, DEFAULT_MAX_GAS_AMOUNT};
use starcoin_types::account_address::AccountAddress;
use starcoin_types::block::BlockNumber;
use starcoin_vm_types::access_path::AccessPath;
use starcoin_vm_types::account_config::{
    genesis_address, ModuleUpgradeStrategy, STC_TOKEN_CODE_STR,
};
use starcoin_vm_types::move_resource::MoveResource;
use starcoin_vm_types::state_store::state_key::StateKey;
use starcoin_vm_types::state_view::{StateReaderExt, StateView};
use starcoin_vm_types::transaction::{
    Package, RawUserTransaction, Transaction, TransactionPayload,
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

    #[clap(long, short = 'o', parse(from_os_str))]
    /// starcoin node db path. like ~/.starcoin/main
    pub to_path: PathBuf,

    #[clap(long, short = 's')]
    pub block_num: Option<BlockNumber>,
}

pub fn force_deploy_output(
    to_dir: PathBuf,
    net: BuiltinNetworkID,
    _block_number: Option<BlockNumber>,
) -> anyhow::Result<()> {
    ::starcoin_logger::init();
    if net != BuiltinNetworkID::Main
        && net != BuiltinNetworkID::Barnard
        && net != BuiltinNetworkID::Halley
    {
        eprintln!("network only support main, barnard, halley");
        return Ok(());
    }
    let network = ChainNetwork::new_builtin(net);
    let db_storage = DBStorage::new(to_dir.join("starcoindb/db"), RocksdbConfig::default(), None)?;
    let storage = Arc::new(Storage::new(StorageInstance::new_cache_and_db_instance(
        CacheStorage::new(None),
        db_storage,
    ))?);
    let (chain_info, _) =
        Genesis::init_and_check_storage(&network, storage.clone(), to_dir.as_ref())?;
    let _chain = BlockChain::new(
        network.time_service(),
        chain_info.head().id(),
        storage.clone(),
        None,
    )
    .expect("create block chain should success.");

    let package_file = "stdlib.blob".to_string();
    let package = FORCE_UPGRADE_PACKAGE
        .get_file(package_file.clone())
        .map(|file| {
            bcs_ext::from_bytes::<Package>(file.contents()).expect("Decode package should success")
        })
        .ok_or_else(|| format_err!("Can not find upgrade package {}", package_file))?;

    let state_root = chain_info.head().state_root();
    let statedb = ChainStateDB::new(storage, Some(state_root));
    let account =
        create_account("7a2042a26e9c36061d6da365657c4a6a832fd25332de7fda5a39456d6023e478")?;
    let addr = AccountAddress::from_hex_literal("0xbe361d5237428276e86a9f5d50726e6c")?;
    let seq_num = statedb.get_sequence_number(addr)?;
    // let time = net.time_service().now_secs() + DEFAULT_EXPIRATION_TIME;

    let time = match net {
        BuiltinNetworkID::Main => {
            // main block num 16912223
            1710453679
        }
        BuiltinNetworkID::Barnard => {
            // main block num 16912223
            1710453679
        }
        _ => {
            // halley block num 109
            1713011341
        }
    };
    println!("time {}", time);
    let txn = account.sign_txn(RawUserTransaction::new(
        addr,
        seq_num,
        TransactionPayload::Package(package),
        DEFAULT_MAX_GAS_AMOUNT,
        1,
        //  net.time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
        time,
        network.chain_id(),
        STC_TOKEN_CODE_STR.to_string(),
    ));
    let upgrade_strategy_path =
        AccessPath::resource_access_path(genesis_address(), ModuleUpgradeStrategy::struct_tag());
    let old_strategy = statedb
        .get_state_value(&StateKey::AccessPath(upgrade_strategy_path.clone()))?
        .expect("get access error");
    println!("before {}", old_strategy[0]);
    statedb
        .set(&upgrade_strategy_path, vec![100])
        .expect("update resource failed");
    let strategy = statedb
        .get_state_value(&StateKey::AccessPath(upgrade_strategy_path))?
        .expect("get access error");
    println!("now {}", strategy[0]);
    let ret = starcoin_executor::execute_transactions(
        &statedb,
        vec![Transaction::UserTransaction(txn)],
        None,
    )?;
    if !ret.is_empty() {
        println!("execute OK");
        println!("{:?}", serde_json::to_string(&ret[0].write_set())?);
        if ret.len() > 1 {
            println!("{:?}", serde_json::to_string(&ret[1].write_set())?);
        }
    }
    Ok(())
}

fn create_account(private_hex: &str) -> anyhow::Result<Account> {
    let bytes = hex::decode(private_hex)?;
    let private_key = Ed25519PrivateKey::try_from(&bytes[..])?;
    let public_key = Ed25519PublicKey::from(&private_key);
    Ok(Account::with_keypair(
        private_key.into(),
        public_key.into(),
        None,
    ))
}
