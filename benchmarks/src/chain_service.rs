// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::random_txn;
use actix::Addr;
use criterion::{BatchSize, Bencher};
use parking_lot::RwLock;
use rand::prelude::*;
use rand::{RngCore, SeedableRng};
use starcoin_account_api::AccountInfo;
use starcoin_bus::BusActor;
use starcoin_chain::{BlockChain, ChainServiceImpl};
use starcoin_config::NodeConfig;
use starcoin_consensus::Consensus;
use starcoin_genesis::Genesis;
use starcoin_storage::Storage;
use starcoin_txpool::{TxPool, TxPoolService};
use starcoin_vm_types::genesis_config::ConsensusStrategy;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use traits::{ChainReader, ReadableChainService};

/// Benchmarking support for chain.
pub struct ChainServiceBencher {
    chain: Arc<RwLock<ChainServiceImpl<TxPoolService>>>,
    config: Arc<NodeConfig>,
    storage: Arc<Storage>,
    block_num: u64,
    account: AccountInfo,
    count: AtomicU64,
}

impl ChainServiceBencher {
    pub fn new(num: Option<u64>, bus: Addr<BusActor>) -> Self {
        let node_config = NodeConfig::random_for_test();
        let node_config = Arc::new(node_config);
        let (storage, startup_info, _) = Genesis::init_storage_for_test(node_config.net())
            .expect("init storage by genesis fail.");

        let txpool = {
            let best_block_id = *startup_info.get_master();
            TxPool::start(node_config.clone(), storage.clone(), best_block_id, bus)
        };
        let chain = ChainServiceImpl::<TxPoolService>::new(
            node_config.clone(),
            startup_info,
            storage.clone(),
            txpool.get_service(),
        )
        .unwrap();
        let miner_account = AccountInfo::random();

        ChainServiceBencher {
            chain: Arc::new(RwLock::new(chain)),
            block_num: match num {
                Some(n) => n,
                None => 100,
            },
            config: node_config,
            storage,
            account: miner_account,
            count: AtomicU64::new(0),
        }
    }

    pub fn execute(&self, proportion: Option<u64>) {
        let mut latest_id = None;
        let mut rng: StdRng = StdRng::from_seed([0; 32]);
        for i in 0..self.block_num {
            let block_chain = BlockChain::new(
                self.config.net().consensus(),
                self.chain.read().get_master().head_block().header().id(),
                self.storage.clone(),
            )
            .unwrap();

            let mut branch_flag = false;
            if let Some(p) = proportion {
                let random = rng.next_u64();
                if (random % p) == 0 {
                    branch_flag = true;
                };
            };
            let parent = if branch_flag && i > 0 {
                latest_id
            } else {
                self.count.fetch_add(1, Ordering::Relaxed);
                None
            };
            let mut txn_vec = Vec::new();
            txn_vec.push(random_txn(self.count.load(Ordering::Relaxed)));
            let block_template = self
                .chain
                .read()
                .create_block_template(self.account.public_key.clone(), parent, txn_vec)
                .unwrap();
            let block = ConsensusStrategy::Dummy
                .create_block(&block_chain, block_template)
                .unwrap();
            latest_id = Some(block.header().parent_hash());
            //self.chain.write().try_connect(block).unwrap();
        }
    }

    fn execute_query(&self, times: u64) {
        let max_num = self.chain.read().master_head_header().number();
        let mut rng = rand::thread_rng();
        for _i in 0..times {
            let number = rng.gen_range(0, max_num);
            assert!(self
                .chain
                .read()
                .master_block_by_number(number)
                .unwrap()
                .is_some());
        }
    }

    pub fn query_bench(&self, b: &mut Bencher, times: u64) {
        b.iter_batched(
            || (self, times),
            |(bench, t)| bench.execute_query(t),
            BatchSize::LargeInput,
        )
    }

    pub fn bench(&self, b: &mut Bencher, proportion: Option<u64>) {
        b.iter_batched(
            || (self, proportion),
            |(bench, p)| bench.execute(p),
            BatchSize::LargeInput,
        )
    }
}
