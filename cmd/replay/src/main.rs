// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_chain::BlockChain;
use starcoin_config::RocksdbConfig;
use starcoin_genesis::Genesis;
use starcoin_storage::cache_storage::CacheStorage;
use starcoin_storage::db_storage::DBStorage;
use starcoin_storage::storage::StorageInstance;
use starcoin_storage::Storage;
use starcoin_vm_types::genesis_config::{BuiltinNetworkID, ChainNetwork};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;
use structopt::StructOpt;
use traits::{ChainReader, ChainWriter};

#[derive(Debug, StructOpt)]
#[structopt(name = "replay")]
pub struct ReplayOpt {
    #[structopt(long, short = "n")]
    /// Chain Network to replay.
    pub net: Option<BuiltinNetworkID>,
    #[structopt(short = "f", long, parse(from_os_str))]
    /// Replay data dir.
    pub from: PathBuf,
    #[structopt(short = "t", long, parse(from_os_str))]
    /// Target dir.
    pub to: PathBuf,
    #[structopt(long, short = "c", default_value = "20000")]
    /// Number of block.
    pub block_num: u64,
}

fn main() {
    let _logger = starcoin_logger::init();
    let opts = ReplayOpt::from_args();

    let network = match opts.net {
        Some(network) => network,
        None => BuiltinNetworkID::Proxima,
    };
    let net = ChainNetwork::new_builtin(network);

    let from_dir = opts.from;
    let block_num = opts.block_num;
    let to_dir = opts.to;

    let db_storage =
        DBStorage::new(from_dir.join("starcoindb/db"), RocksdbConfig::default()).unwrap();

    let storage = Arc::new(
        Storage::new(StorageInstance::new_cache_and_db_instance(
            CacheStorage::new(),
            db_storage,
        ))
        .unwrap(),
    );
    let (startup_info, _) =
        Genesis::init_and_check_storage(&net, storage.clone(), from_dir.as_ref())
            .expect("init storage by genesis fail.");
    let chain = BlockChain::new(net.time_service(), startup_info.main, storage)
        .expect("create block chain should success.");
    //read from first chain
    let begin = SystemTime::now();
    let mut block_vec = vec![];
    for i in 1..block_num {
        block_vec.push(chain.get_block_by_number(i).unwrap().unwrap());
    }
    let use_time = SystemTime::now().duration_since(begin).unwrap();
    println!("read use time: {:?}", use_time.as_nanos());

    let storage2 = Arc::new(
        Storage::new(StorageInstance::new_cache_and_db_instance(
            CacheStorage::new(),
            DBStorage::new(to_dir.join("starcoindb"), RocksdbConfig::default()).unwrap(),
        ))
        .unwrap(),
    );
    let (startup_info2, _) =
        Genesis::init_and_check_storage(&net, storage2.clone(), to_dir.as_ref())
            .expect("init storage by genesis fail.");

    let mut chain2 = BlockChain::new(net.time_service(), startup_info2.main, storage2)
        .expect("create block chain should success.");
    let begin = SystemTime::now();
    for block in block_vec {
        chain2.apply(block).unwrap();
    }
    let use_time = SystemTime::now().duration_since(begin).unwrap();
    println!("apply use time: {:?}", use_time.as_nanos());
}
