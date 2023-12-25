use super::ghostdag::protocol::GhostdagManager;
use super::reachability::{inquirer, reachability_service::MTReachabilityService};
use super::types::ghostdata::GhostdagData;
use crate::consensusdb::prelude::{FlexiDagStorageConfig, StoreError};
use crate::consensusdb::schemadb::GhostdagStoreReader;
use crate::consensusdb::{
    prelude::FlexiDagStorage,
    schemadb::{
        DbGhostdagStore, DbHeadersStore, DbReachabilityStore, DbRelationsStore, GhostdagStore,
        HeaderStore, ReachabilityStoreReader, RelationsStore, RelationsStoreReader,
    },
};
use anyhow::{bail, Ok};
use parking_lot::RwLock;
use starcoin_config::{temp_dir, ChainNetworkID, RocksdbConfig};
use starcoin_crypto::{HashValue as Hash, HashValue};
use starcoin_storage::Store;
use starcoin_types::block::{BlockHeader, BlockNumber, TEST_FLEXIDAG_FORK_HEIGHT, DEV_FLEXIDAG_FORK_HEIGHT, HALLEY_FLEXIDAG_FORK_HEIGHT, PROXIMA_FLEXIDAG_FORK_HEIGHT, BARNARD_FLEXIDAG_FORK_HEIGHT, MAIN_FLEXIDAG_FORK_HEIGHT};
use starcoin_types::{
    blockhash::{BlockHashes, KType},
    consensus_header::ConsensusHeader,
};
use starcoin_vm_types::genesis_config::ChainId;
use std::path::{self, Path};
use std::sync::Arc;

pub type DbGhostdagManager = GhostdagManager<
    DbGhostdagStore,
    DbRelationsStore,
    MTReachabilityService<DbReachabilityStore>,
    DbHeadersStore,
>;

#[derive(Clone)]
pub struct BlockDAG {
    pub storage: FlexiDagStorage,
    ghostdag_manager: DbGhostdagManager,
}

impl BlockDAG {
    pub fn new(k: KType, db: FlexiDagStorage) -> Self {
        let ghostdag_store = db.ghost_dag_store.clone();
        let header_store = db.header_store.clone();
        let relations_store = db.relations_store.clone();
        let reachability_store = db.reachability_store.clone();
        let reachability_service =
            MTReachabilityService::new(Arc::new(RwLock::new(reachability_store)));
        let ghostdag_manager = DbGhostdagManager::new(
            k,
            ghostdag_store.clone(),
            relations_store.clone(),
            header_store.clone(),
            reachability_service,
        );

        Self {
            ghostdag_manager,
            storage: db,
        }
    }
    pub fn create_for_testing() -> anyhow::Result<Self> {
        let dag_storage =
            FlexiDagStorage::create_from_path(temp_dir(), FlexiDagStorageConfig::default())?;
        Ok(BlockDAG::new(16, dag_storage))
    }

    pub fn new_by_config(db_path: &Path) -> anyhow::Result<BlockDAG> {
        let config = FlexiDagStorageConfig::create_with_params(1, RocksdbConfig::default());
        let db = FlexiDagStorage::create_from_path(db_path, config)?;
        let dag = Self::new(16, db);
        Ok(dag)
    }

    pub fn try_init_with_storage(
        storage: Arc<dyn Store>,
    ) -> anyhow::Result<Self> {
        let startup_info = storage
            .get_startup_info()?
            .expect("startup info must exist");

        let block_header = storage
            .get_block_header_by_hash(startup_info.get_main().clone())?
            .expect("the genesis block in dag accumulator must none be none");

        let dag = Self::new_by_config(
            storage.path().join("flexidag").as_path(),
        )?;

        let fork_height = block_header.dag_fork_height();

        if block_header.number() < fork_height {
            Ok(dag)
        } else if block_header.number() == fork_height {
            dag.init_with_genesis(block_header)?;
            Ok(dag)
        } else {
            Ok(dag)
        }
    }

    pub fn dag_fork_height_with_net(net: ChainId) -> BlockNumber {
        if net.is_barnard() {
            BARNARD_FLEXIDAG_FORK_HEIGHT
        } else if net.is_dev() {
            DEV_FLEXIDAG_FORK_HEIGHT
        } else if net.is_halley() {
            HALLEY_FLEXIDAG_FORK_HEIGHT
        } else if net.is_main() {
            MAIN_FLEXIDAG_FORK_HEIGHT
        } else if net.is_test() {
            TEST_FLEXIDAG_FORK_HEIGHT
        } else if net.is_proxima() {
            PROXIMA_FLEXIDAG_FORK_HEIGHT
        } else {
            DEV_FLEXIDAG_FORK_HEIGHT
        }
    }

    pub fn has_dag_block(&self, hash: Hash) -> anyhow::Result<bool> {
        Ok(self.storage.header_store.has(hash)?)
    }

    pub fn init_with_genesis(&self, genesis: BlockHeader) -> anyhow::Result<()> {
        let origin = genesis.parent_hash();

        if self.storage.relations_store.has(origin)? {
            return Ok(());
        };
        inquirer::init(&mut self.storage.reachability_store.clone(), origin)?;
        self.storage
            .relations_store
            .insert(origin, BlockHashes::new(vec![]))?;
        self.commit(genesis)?;
        Ok(())
    }
    pub fn ghostdata(&self, parents: &[HashValue]) -> GhostdagData {
        self.ghostdag_manager.ghostdag(parents)
    }

    pub fn ghostdata_by_hash(&self, hash: HashValue) -> anyhow::Result<Option<Arc<GhostdagData>>> {
        match self.storage.ghost_dag_store.get_data(hash) {
            Result::Ok(value) => Ok(Some(value)),
            Err(StoreError::KeyNotFound(_)) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn commit(&self, header: BlockHeader) -> anyhow::Result<()> {
        // Generate ghostdag data
        let parents = header.parents();
        let ghostdata = self.ghostdata_by_hash(header.id())?.unwrap_or_else(|| {
            Arc::new(if header.is_dag_genesis() {
                self.ghostdag_manager.genesis_ghostdag_data(&header)
            } else {
                self.ghostdag_manager.ghostdag(&parents)
            })
        });
        // Store ghostdata
        self.storage
            .ghost_dag_store
            .insert(header.id(), ghostdata.clone())?;

        // Update reachability store
        let mut reachability_store = self.storage.reachability_store.clone();
        let mut merge_set = ghostdata
            .unordered_mergeset_without_selected_parent()
            .filter(|hash| self.storage.reachability_store.has(*hash).unwrap());
        inquirer::add_block(
            &mut reachability_store,
            header.id(),
            ghostdata.selected_parent,
            &mut merge_set,
        )?;
        // store relations
        self.storage
            .relations_store
            .insert(header.id(), BlockHashes::new(parents))?;
        // Store header store
        let _ = self
            .storage
            .header_store
            .insert(header.id(), Arc::new(header.to_owned()), 0)?;
        return Ok(());
    }

    pub fn get_parents(&self, hash: Hash) -> anyhow::Result<Vec<Hash>> {
        match self.storage.relations_store.get_parents(hash) {
            anyhow::Result::Ok(parents) => anyhow::Result::Ok((*parents).clone()),
            Err(error) => {
                println!("failed to get parents by hash: {}", error.to_string());
                bail!("failed to get parents by hash: {}", error.to_string());
            }
        }
    }

    pub fn get_children(&self, hash: Hash) -> anyhow::Result<Vec<Hash>> {
        match self.storage.relations_store.get_children(hash) {
            anyhow::Result::Ok(children) => anyhow::Result::Ok((*children).clone()),
            Err(error) => {
                println!("failed to get parents by hash: {}", error.to_string());
                bail!("failed to get parents by hash: {}", error.to_string());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consensusdb::prelude::FlexiDagStorageConfig;
    use starcoin_config::RocksdbConfig;
    use starcoin_types::block::{BlockHeader, BlockHeaderBuilder};
    use std::{env, fs};

    fn build_block_dag(k: KType) -> BlockDAG {
        let db_path = env::temp_dir().join("smolstc");
        println!("db path:{}", db_path.to_string_lossy());
        if db_path
            .as_path()
            .try_exists()
            .unwrap_or_else(|_| panic!("Failed to check {db_path:?}"))
        {
            fs::remove_dir_all(db_path.as_path()).expect("Failed to delete temporary directory");
        }
        let config = FlexiDagStorageConfig::create_with_params(1, RocksdbConfig::default());
        let db = FlexiDagStorage::create_from_path(db_path, config)
            .expect("Failed to create flexidag storage");
        let dag = BlockDAG::new(k, db);
        return dag;
    }

    #[test]
    fn test_dag_0() {
        //let dag = build_block_dag(16);
        let dag = BlockDAG::create_for_testing().unwrap();
        let genesis = BlockHeader::dag_genesis_random()
            .as_builder()
            .with_difficulty(0.into())
            .build();

        let mut parents_hash = vec![genesis.id()];
        dag.init_with_genesis(genesis.to_owned()).unwrap();

        for _ in 0..10 {
            let header_builder = BlockHeaderBuilder::random();
            let header = header_builder
                .with_parents_hash(Some(parents_hash.clone()))
                .build();
            parents_hash = vec![header.id()];
            dag.commit(header.to_owned()).unwrap();
            let ghostdata = dag.ghostdata_by_hash(header.id()).unwrap().unwrap();
            println!("{:?},{:?}", header, ghostdata);
        }
    }

    #[test]
    fn test_dag_1() {
        let genesis = BlockHeader::dag_genesis_random()
            .as_builder()
            .with_difficulty(0.into())
            .build();
        let block1 = BlockHeaderBuilder::random()
            .with_difficulty(1.into())
            .with_parents_hash(Some(vec![genesis.id()]))
            .build();
        let block2 = BlockHeaderBuilder::random()
            .with_difficulty(2.into())
            .with_parents_hash(Some(vec![genesis.id()]))
            .build();
        let block3_1 = BlockHeaderBuilder::random()
            .with_difficulty(1.into())
            .with_parents_hash(Some(vec![genesis.id()]))
            .build();
        let block3 = BlockHeaderBuilder::random()
            .with_difficulty(3.into())
            .with_parents_hash(Some(vec![block3_1.id()]))
            .build();
        let block4 = BlockHeaderBuilder::random()
            .with_difficulty(4.into())
            .with_parents_hash(Some(vec![block1.id(), block2.id()]))
            .build();
        let block5 = BlockHeaderBuilder::random()
            .with_difficulty(4.into())
            .with_parents_hash(Some(vec![block2.id(), block3.id()]))
            .build();
        let block6 = BlockHeaderBuilder::random()
            .with_difficulty(5.into())
            .with_parents_hash(Some(vec![block4.id(), block5.id()]))
            .build();
        let mut latest_id = block6.id();
        let genesis_id = genesis.id();
        let dag = build_block_dag(3);
        let expect_selected_parented = vec![block5.id(), block3.id(), block3_1.id(), genesis_id];
        dag.init_with_genesis(genesis).unwrap();

        dag.commit(block1).unwrap();
        dag.commit(block2).unwrap();
        dag.commit(block3_1).unwrap();
        dag.commit(block3).unwrap();
        dag.commit(block4).unwrap();
        dag.commit(block5).unwrap();
        dag.commit(block6).unwrap();
        let mut count = 0;
        while latest_id != genesis_id && count < 4 {
            let ghostdata = dag.ghostdata_by_hash(latest_id).unwrap().unwrap();
            latest_id = ghostdata.selected_parent;
            assert_eq!(expect_selected_parented[count], latest_id);
            count += 1;
        }
    }
}
