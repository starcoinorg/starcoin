// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use criterion::{BatchSize, Bencher};
use parking_lot::RwLock;
use rand::prelude::*;
use starcoin_account_api::AccountInfo;
use starcoin_chain::BlockChain;
use starcoin_consensus::Consensus;
use starcoin_genesis::Genesis;
use starcoin_vm_types::chain_config::{ChainNetwork, ConsensusStrategy};
use std::ops::Deref;
use std::sync::Arc;
use traits::{ChainReader, ChainWriter};

/// Benchmarking support for chain.
pub struct ChainBencher {
    chain: Arc<RwLock<BlockChain>>,
    block_num: u64,
    account: AccountInfo,
}

impl ChainBencher {
    pub fn new(num: Option<u64>) -> Self {
        let net = ChainNetwork::Test;
        let (storage, startup_info, _) =
            Genesis::init_storage_for_test(net).expect("init storage by genesis fail.");

        let chain = BlockChain::new(net, startup_info.master, storage, None)
            .expect("create block chain should success.");
        let miner_account = AccountInfo::random();

        ChainBencher {
            chain: Arc::new(RwLock::new(chain)),
            block_num: match num {
                Some(n) => n,
                None => 100,
            },
            account: miner_account,
        }
    }

    pub fn execute(&self, _proportion: Option<u64>) {
        for _i in 0..self.block_num {
            //let mut txn_vec = Vec::new();
            //txn_vec.push(random_txn(self.count.load(Ordering::Relaxed)));
            let (block_template, _) = self
                .chain
                .read()
                .create_block_template(
                    *self.account.address(),
                    Some(self.account.get_auth_key().prefix().to_vec()),
                    None,
                    vec![],
                    vec![],
                    None,
                )
                .unwrap();
            let block = ConsensusStrategy::Dummy
                .create_block(self.chain.read().deref(), block_template)
                .unwrap();
            self.chain.write().apply(block).unwrap();
        }
    }

    fn execute_query(&self, times: u64) {
        let max_num = self.chain.read().current_header().number();
        let mut rng = rand::thread_rng();
        for _i in 0..times {
            let number = rng.gen_range(0, max_num);
            assert!(self
                .chain
                .read()
                .get_block_by_number(number)
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
