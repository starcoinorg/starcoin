use super::reachability::{inquirer, reachability_service::MTReachabilityService};
use super::types::ghostdata::GhostdagData;
use crate::block_dag_config::{BlockDAGConfigMock, BlockDAGType};
use crate::consensusdb::consenses_state::{DagState, DagStateReader, DagStateStore};
use crate::consensusdb::prelude::{FlexiDagStorageConfig, StoreError};
use crate::consensusdb::schemadb::{GhostdagStoreReader, ReachabilityStore, REINDEX_ROOT_KEY};
use crate::consensusdb::{
    prelude::FlexiDagStorage,
    schemadb::{
        DbGhostdagStore, DbHeadersStore, DbReachabilityStore, DbRelationsStore, GhostdagStore,
        HeaderStore, ReachabilityStoreReader, RelationsStore, RelationsStoreReader,
    },
};
use crate::ghostdag::protocol::GhostdagManager;
use crate::{process_key_already_error, reachability};
use anyhow::{anyhow, bail, Ok};
use bcs_ext::BCSCodec;
use parking_lot::RwLock;
use starcoin_config::{temp_dir, RocksdbConfig};
use starcoin_crypto::{HashValue as Hash, HashValue};
use starcoin_logger::prelude::info;
use starcoin_types::block::{BlockHeader, TEST_FLEXIDAG_FORK_HEIGHT_NEVER_REACH};
use starcoin_types::{
    blockhash::{BlockHashes, KType},
    consensus_header::ConsensusHeader,
};
use std::path::Path;
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
    dag_config: BlockDAGType,
}

impl BlockDAG {
    pub fn new_with_type(k: KType, db: FlexiDagStorage, dag_config: BlockDAGType) -> Self {
        let ghostdag_store = db.ghost_dag_store.clone();
        let header_store = db.header_store.clone();
        let relations_store = db.relations_store.clone();
        let reachability_store = db.reachability_store.clone();
        let reachability_service =
            MTReachabilityService::new(Arc::new(RwLock::new(reachability_store)));
        let ghostdag_manager = DbGhostdagManager::new(
            k,
            ghostdag_store,
            relations_store,
            header_store,
            reachability_service,
        );

        Self {
            ghostdag_manager,
            storage: db,
            dag_config,
        }
    }

    pub fn new(k: KType, db: FlexiDagStorage) -> Self {
        Self::new_with_type(k, db, BlockDAGType::BlockDAGFormal)
    }
    pub fn create_for_testing() -> anyhow::Result<Self> {
        let dag_storage =
            FlexiDagStorage::create_from_path(temp_dir(), FlexiDagStorageConfig::default())?;
        Ok(BlockDAG::new_with_type(
            8,
            dag_storage,
            BlockDAGType::BlockDAGTestMock(BlockDAGConfigMock {
                fork_number: TEST_FLEXIDAG_FORK_HEIGHT_NEVER_REACH,
            }),
        ))
    }

    pub fn create_for_testing_mock(config: BlockDAGConfigMock) -> anyhow::Result<Self> {
        let dag_storage =
            FlexiDagStorage::create_from_path(temp_dir(), FlexiDagStorageConfig::default())?;
        Ok(BlockDAG::new_with_type(
            8,
            dag_storage,
            BlockDAGType::BlockDAGTestMock(config),
        ))
    }

    pub fn new_by_config(db_path: &Path) -> anyhow::Result<BlockDAG> {
        let config = FlexiDagStorageConfig::create_with_params(1, RocksdbConfig::default());
        let db = FlexiDagStorage::create_from_path(db_path, config)?;
        let dag = Self::new(8, db);
        Ok(dag)
    }

    pub fn block_dag_config(&self) -> BlockDAGType {
        self.dag_config.clone()
    }

    pub fn has_dag_block(&self, hash: Hash) -> anyhow::Result<bool> {
        Ok(self.storage.header_store.has(hash)?)
    }

    pub fn check_ancestor_of(&self, ancestor: Hash, descendant: Vec<Hash>) -> anyhow::Result<bool> {
        self.ghostdag_manager
            .check_ancestor_of(ancestor, descendant)
    }

    pub fn init_with_genesis(&mut self, genesis: BlockHeader) -> anyhow::Result<HashValue> {
        let genesis_id = genesis.id();
        let origin = genesis.parent_hash();

        let real_origin = Hash::sha3_256_of(&[origin, genesis_id].encode()?);

        if self.storage.relations_store.has(real_origin)? {
            return Ok(real_origin);
        }
        inquirer::init(&mut self.storage.reachability_store.clone(), real_origin)?;

        self.storage
            .relations_store
            .insert(real_origin, BlockHashes::new(vec![]))?;
        // self.storage
        //     .relations_store
        //     .insert(origin, BlockHashes::new(vec![]))?;
        self.commit(genesis, real_origin)?;
        self.save_dag_state(
            genesis_id,
            DagState {
                tips: vec![genesis_id],
            },
        )?;
        Ok(real_origin)
    }
    pub fn ghostdata(&self, parents: &[HashValue]) -> Result<GhostdagData, StoreError> {
        self.ghostdag_manager.ghostdag(parents)
    }

    pub fn ghostdata_by_hash(&self, hash: HashValue) -> anyhow::Result<Option<Arc<GhostdagData>>> {
        match self.storage.ghost_dag_store.get_data(hash) {
            Result::Ok(value) => Ok(Some(value)),
            Err(StoreError::KeyNotFound(_)) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn set_reindex_root(&mut self, hash: HashValue) -> anyhow::Result<()> {
        self.storage.reachability_store.set_reindex_root(hash)?;
        Ok(())
    }

    fn commit_genesis(&self, genesis: BlockHeader) -> anyhow::Result<()> {
        self.commit_inner(genesis, true)
    }

    pub fn commit(&self, header: BlockHeader) -> anyhow::Result<()> {
        self.commit_inner(header, false)
    }
    
    pub fn commit_inner(&mut self, header: BlockHeader, origin: HashValue, is_dag_genesis: bool) -> anyhow::Result<()> {
        // Generate ghostdag data
        let parents = header.parents();
        let ghostdata = match self.ghostdata_by_hash(header.id())? {
            None => {
                if is_dag_genesis {
                    Arc::new(self.ghostdag_manager.genesis_ghostdag_data(&header))
                } else {
                    let ghostdata = self.ghostdag_manager.ghostdag(&parents)?;
                    Arc::new(ghostdata)
                }
            }
            Some(ghostdata) => ghostdata,
        };
        // Store ghostdata
        process_key_already_error(
            self.storage
                .ghost_dag_store
                .insert(header.id(), ghostdata.clone()),
        )?;

        // Update reachability store
        let mut reachability_store = self.storage.reachability_store.clone();
        let mut merge_set = ghostdata
            .unordered_mergeset_without_selected_parent()
            .filter(|hash| self.storage.reachability_store.has(*hash).unwrap());
        match inquirer::add_block(
            &mut reachability_store,
            header.id(),
            ghostdata.selected_parent,
            &mut merge_set,
        ) {
            Result::Ok(_) => (),
            Err(reachability::ReachabilityError::DataInconsistency) => {
                let _future_covering_set =
                    reachability_store.get_future_covering_set(header.id())?;
                info!(
                    "the key {:?} was already processed, original error message: {:?}",
                    header.id(),
                    reachability::ReachabilityError::DataInconsistency
                );
            }
            Err(reachability::ReachabilityError::StoreError(StoreError::KeyNotFound(msg))) => {
                if msg == *REINDEX_ROOT_KEY.to_string() {
                    info!(
                        "the key {:?} was already processed, original error message: {:?}",
                        header.id(),
                        reachability::ReachabilityError::StoreError(StoreError::KeyNotFound(
                            REINDEX_ROOT_KEY.to_string()
                        ))
                    );
                    info!("now set the reindex key to origin: {:?}", origin);
                    // self.storage.reachability_store.set_reindex_root(origin)?;
                    self.set_reindex_root(origin)?;
                    bail!(
                        "failed to add a block when committing, e: {:?}",
                        reachability::ReachabilityError::StoreError(StoreError::KeyNotFound(msg))
                    );
                } else {
                    bail!(
                        "failed to add a block when committing, e: {:?}",
                        reachability::ReachabilityError::StoreError(StoreError::KeyNotFound(msg))
                    );
                }
            }
            Err(e) => {
                bail!("failed to add a block when committing, e: {:?}", e);
            }
        }

        // store relations
        if header.is_dag_genesis() {
            let origin = header.parent_hash();
            let real_origin = Hash::sha3_256_of(&[origin, header.id()].encode()?);
            process_key_already_error(
                self.storage
                    .relations_store
                    .insert(header.id(), BlockHashes::new(vec![real_origin])),
            )?;
        } else {
            process_key_already_error(
                self.storage
                    .relations_store
                    .insert(header.id(), BlockHashes::new(parents)),
            )?;
        }
        // Store header store
        process_key_already_error(self.storage.header_store.insert(
            header.id(),
            Arc::new(header),
            0,
        ))?;
        Ok(())
    }

    pub fn get_parents(&self, hash: Hash) -> anyhow::Result<Vec<Hash>> {
        match self.storage.relations_store.get_parents(hash) {
            anyhow::Result::Ok(parents) => anyhow::Result::Ok((*parents).clone()),
            Err(error) => {
                bail!("failed to get parents by hash: {}", error);
            }
        }
    }

    pub fn get_children(&self, hash: Hash) -> anyhow::Result<Vec<Hash>> {
        match self.storage.relations_store.get_children(hash) {
            anyhow::Result::Ok(children) => anyhow::Result::Ok((*children).clone()),
            Err(error) => {
                bail!("failed to get parents by hash: {}", error);
            }
        }
    }
    
    pub fn get_dag_state(&self, hash: Hash) -> anyhow::Result<DagState> {
        Ok(self.storage.state_store.get_state(hash)?)
    }

    pub fn save_dag_state(&self, hash: Hash, state: DagState) -> anyhow::Result<()> {
        self.storage.state_store.insert(hash, state)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consensusdb::prelude::FlexiDagStorageConfig;
    use starcoin_config::RocksdbConfig;
    use starcoin_types::block::{
        BlockHeader, BlockHeaderBuilder, TEST_FLEXIDAG_FORK_HEIGHT_FOR_DAG,
    };
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
        BlockDAG::new(k, db)
    }

    #[test]
    fn test_dag_0() {
        let dag = BlockDAG::create_for_testing().unwrap();
        let genesis = BlockHeader::dag_genesis_random(TEST_FLEXIDAG_FORK_HEIGHT_FOR_DAG)
            .as_builder()
            .with_difficulty(0.into())
            .build();

        let mut parents_hash = vec![genesis.id()];
        dag.init_with_genesis(genesis).unwrap();

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
        let genesis = BlockHeader::dag_genesis_random(TEST_FLEXIDAG_FORK_HEIGHT_FOR_DAG)
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

    #[tokio::test]
    async fn test_with_spawn() {
        use starcoin_types::block::{BlockHeader, BlockHeaderBuilder};
        let genesis = BlockHeader::dag_genesis_random(TEST_FLEXIDAG_FORK_HEIGHT_FOR_DAG)
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
        let dag = BlockDAG::create_for_testing().unwrap();
        dag.init_with_genesis(genesis).unwrap();
        dag.commit(block1.clone()).unwrap();
        dag.commit(block2.clone()).unwrap();
        let block3 = BlockHeaderBuilder::random()
            .with_difficulty(3.into())
            .with_parents_hash(Some(vec![block1.id(), block2.id()]))
            .build();
        let mut handles = vec![];
        for _i in 1..100 {
            let dag_clone = dag.clone();
            let block_clone = block3.clone();
            let handle = tokio::task::spawn_blocking(move || {
                let _ = dag_clone.commit(block_clone);
            });
            handles.push(handle);
        }
        for handle in handles {
            handle.await.unwrap();
        }
        let mut child = dag.get_children(block1.id()).unwrap();
        assert_eq!(child.pop().unwrap(), block3.id());
        assert_eq!(child.len(), 0);
    }
}
