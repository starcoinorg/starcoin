
// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use parking_lot::RwLock;
use starcoin_account_api::AccountInfo;
use starcoin_chain::BlockChain;
use starcoin_config::temp_path;
use starcoin_consensus::Consensus;
use starcoin_genesis::Genesis;
use starcoin_storage::cache_storage::CacheStorage;
use starcoin_storage::db_storage::DBStorage;
use starcoin_storage::storage::StorageInstance;
use starcoin_storage::Storage;
use starcoin_vm_types::genesis_config::{ChainNetwork, ConsensusStrategy};
use std::ops::Deref;
use std::sync::Arc;
use traits::ChainWriter;

/// Benchmarking support for chain.
pub struct ChainBencher {
    net: ChainNetwork,
    chain: Arc<RwLock<BlockChain>>,
    block_num: u64,
    account: AccountInfo,
}

impl ChainBencher {
    pub fn new(num: Option<u64>) -> Self {
        let net = ChainNetwork::new_test();
        let temp_path = temp_path();
        let storage = Arc::new(
            Storage::new(StorageInstance::new_cache_and_db_instance(
                CacheStorage::new(),
                DBStorage::new(temp_path.path().join("starcoindb")).unwrap(),
            ))
                .unwrap(),
        );
        let (startup_info, _) =
            Genesis::init_and_check_storage(&net, storage.clone(), temp_path.path())
                .expect("init storage by genesis fail.");

        let chain = BlockChain::new(net.time_service(), startup_info.main, storage)
            .expect("create block chain should success.");
        let miner_account = AccountInfo::random();

        ChainBencher {
            net,
            chain: Arc::new(RwLock::new(chain)),
            block_num: match num {
                Some(n) => n,
                None => 100,
            },
            account: miner_account,
        }
    }

    pub fn execute(&self) {
        for i in 0..self.block_num {
            //let mut txn_vec = Vec::new();
            //txn_vec.push(random_txn(self.count.load(Ordering::Relaxed)));
            let (block_template, _) = self
                .chain
                .read()
                .create_block_template(
                    *self.account.address(),
                    Some(self.account.public_key.auth_key()),
                    None,
                    vec![],
                    vec![],
                    None,
                )
                .unwrap();
            let block = ConsensusStrategy::Dummy
                .create_block(
                    self.chain.read().deref(),
                    block_template,
                    self.net.time_service().as_ref(),
                )
                .unwrap();
            self.chain.write().apply(block).unwrap();
            println!("apply block {}", i);
        }
    }

}


