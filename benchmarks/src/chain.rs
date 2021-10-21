// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use criterion::{BatchSize, Bencher};
use parking_lot::RwLock;
use rand::prelude::*;
use starcoin_account_api::AccountInfo;
use starcoin_chain::BlockChain;
use starcoin_chain::{ChainReader, ChainWriter};
use starcoin_config::{temp_path, ChainNetwork, DataDirPath, RocksdbConfig};
use starcoin_consensus::Consensus;
use starcoin_genesis::Genesis;
use starcoin_storage::cache_storage::CacheStorage;
use starcoin_storage::db_storage::DBStorage;
use starcoin_storage::storage::StorageInstance;
use starcoin_storage::Storage;
use starcoin_vm_types::genesis_config::ConsensusStrategy;
use std::sync::Arc;

/// Benchmarking support for chain.
pub struct ChainBencher {
    net: ChainNetwork,
    chain: Arc<RwLock<BlockChain>>,
    block_num: u64,
    account: AccountInfo,
    temp_path: DataDirPath,
}

impl ChainBencher {
    pub fn new(num: Option<u64>) -> Self {
        let net = ChainNetwork::new_test();
        let temp_path = temp_path();
        let storage = Arc::new(
            Storage::new(StorageInstance::new_cache_and_db_instance(
                CacheStorage::new(None),
                DBStorage::new(
                    temp_path.path().join("starcoindb"),
                    RocksdbConfig::default(),
                    None,
                )
                .unwrap(),
            ))
            .unwrap(),
        );
        let (chain_info, _) =
            Genesis::init_and_check_storage(&net, storage.clone(), temp_path.path())
                .expect("init storage by genesis fail.");

        let chain = BlockChain::new(net.time_service(), chain_info.head().id(), storage, None)
            .expect("create block chain should success.");
        let miner_account = AccountInfo::random();

        ChainBencher {
            net,
            chain: Arc::new(RwLock::new(chain)),
            block_num: num.unwrap_or(100),
            account: miner_account,
            temp_path,
        }
    }

    pub fn execute(&self) {
        for _i in 0..self.block_num {
            //let mut txn_vec = Vec::new();
            //txn_vec.push(random_txn(self.count.load(Ordering::Relaxed)));
            let (block_template, _) = self
                .chain
                .read()
                .create_block_template(*self.account.address(), None, vec![], vec![], None)
                .unwrap();
            let block = ConsensusStrategy::Dummy
                .create_block(block_template, self.net.time_service().as_ref())
                .unwrap();
            self.chain.write().apply(block).unwrap();
        }
    }

    fn execute_query(&self, times: u64) {
        let max_num = self.chain.read().current_header().number();
        let mut rng = rand::thread_rng();
        for _i in 0..times {
            let number = rng.gen_range(0..max_num);
            let block = self.chain.read().get_block_by_number(number).unwrap();
            assert!(block.is_some());
            // get block and try to use it.
            let block = block.unwrap();
            let _id = block.id();
        }
    }

    pub fn query_bench(&self, b: &mut Bencher, times: u64) {
        b.iter_batched(
            || (self, times),
            |(bench, t)| bench.execute_query(t),
            BatchSize::LargeInput,
        )
    }

    pub fn bench(&self, b: &mut Bencher) {
        b.iter_batched(|| self, |bench| bench.execute(), BatchSize::LargeInput)
    }
}

impl Clone for ChainBencher {
    fn clone(&self) -> Self {
        Self {
            net: self.net.clone(),
            chain: self.chain.clone(),
            block_num: self.block_num,
            account: self.account.clone(),
            temp_path: self.temp_path.clone(),
        }
    }
}
