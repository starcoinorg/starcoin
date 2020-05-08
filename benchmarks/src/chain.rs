use crate::random_txn;
use actix::Addr;
use criterion::{BatchSize, Bencher};
use parking_lot::RwLock;
use rand::{RngCore, SeedableRng, StdRng};
use starcoin_bus::BusActor;
use starcoin_chain::{
    to_block_chain_collection, BlockChain, BlockChainCollection, ChainServiceImpl,
};
use starcoin_config::NodeConfig;
use starcoin_consensus::dummy::DummyConsensus;
use starcoin_genesis::Genesis;
use starcoin_sync_api::SyncMetadata;
use starcoin_txpool::TxPoolRef;
use starcoin_wallet_api::WalletAccount;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use storage::cache_storage::CacheStorage;
use storage::storage::StorageInstance;
use storage::Storage;
use traits::{ChainService, Consensus};

/// Benchmarking support for chain.
pub struct ChainBencher {
    chain: Arc<RwLock<ChainServiceImpl<DummyConsensus, Storage, TxPoolRef>>>,
    collection: Arc<BlockChainCollection<DummyConsensus, Storage, TxPoolRef>>,
    config: Arc<NodeConfig>,
    storage: Arc<Storage>,
    txpool: TxPoolRef,
    block_num: u64,
    account: WalletAccount,
    count: AtomicU64,
}

impl ChainBencher {
    pub fn new(num: Option<u64>, bus: Addr<BusActor>) -> Self {
        let node_config = NodeConfig::random_for_test();
        let node_config = Arc::new(node_config);
        let storage = Arc::new(
            Storage::new(StorageInstance::new_cache_instance(CacheStorage::new())).unwrap(),
        );
        let genesis = Genesis::build(node_config.net()).unwrap();
        let startup_info = genesis.execute(storage.clone()).unwrap();

        let txpool = {
            let best_block_id = startup_info.master.get_head();
            TxPoolRef::start(
                node_config.tx_pool.clone(),
                storage.clone(),
                best_block_id,
                bus.clone(),
            )
        };
        let sync_metadata = SyncMetadata::new(node_config.clone(), bus.clone());
        let chain = ChainServiceImpl::<DummyConsensus, Storage, TxPoolRef>::new(
            node_config.clone(),
            startup_info,
            storage.clone(),
            None,
            txpool.clone(),
            bus,
            sync_metadata,
        )
        .unwrap();
        let startup_info = chain.master_startup_info();
        let collection = to_block_chain_collection(
            node_config.clone(),
            startup_info,
            storage.clone(),
            txpool.clone(),
        )
        .unwrap();
        let miner_account = WalletAccount::random();

        ChainBencher {
            chain: Arc::new(RwLock::new(chain)),
            block_num: match num {
                Some(n) => n,
                None => 100,
            },
            config: node_config,
            storage,
            txpool,
            collection,
            account: miner_account,
            count: AtomicU64::new(0),
        }
    }

    fn execute(&self, proportion: Option<u64>) {
        let mut latest_id = None;
        let mut rng: StdRng = StdRng::from_seed([0; 32]);
        for i in 0..self.block_num {
            let block_chain = BlockChain::<DummyConsensus, Storage, TxPoolRef>::new(
                self.config.clone(),
                self.collection.get_master_chain_info(),
                self.storage.clone(),
                self.txpool.clone(),
                Arc::downgrade(&self.collection),
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
                .create_block_template(
                    *self.account.address(),
                    Some(self.account.get_auth_key().prefix().to_vec()),
                    parent,
                    txn_vec,
                )
                .unwrap();
            let block =
                DummyConsensus::create_block(self.config.clone(), &block_chain, block_template)
                    .unwrap();
            latest_id = Some(block.header().parent_hash());
            self.chain
                .write()
                .try_connect(block, false)
                .unwrap()
                .unwrap();
        }
    }

    pub fn bench(&self, b: &mut Bencher, proportion: Option<u64>) {
        b.iter_batched(
            || (self, proportion),
            |(bench, p)| bench.execute(p),
            BatchSize::LargeInput,
        )
    }
}
