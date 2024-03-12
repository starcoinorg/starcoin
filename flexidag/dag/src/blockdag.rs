use super::reachability::{inquirer, reachability_service::MTReachabilityService};
use super::types::ghostdata::GhostdagData;
use crate::consensusdb::consenses_state::{DagState, DagStateReader, DagStateStore};
use crate::consensusdb::prelude::{FlexiDagStorageConfig, StoreError};
use crate::consensusdb::schemadb::GhostdagStoreReader;
use crate::consensusdb::{
    prelude::FlexiDagStorage,
    schemadb::{
        DbGhostdagStore, DbHeadersStore, DbReachabilityStore, DbRelationsStore, GhostdagStore,
        HeaderStore, ReachabilityStoreReader, RelationsStore, RelationsStoreReader,
    },
};
use crate::ghostdag::protocol::GhostdagManager;
use anyhow::{bail, Ok};
use bcs_ext::BCSCodec;
use parking_lot::RwLock;
use starcoin_config::{temp_dir, RocksdbConfig};
use starcoin_crypto::{HashValue as Hash, HashValue};
use starcoin_types::block::BlockHeader;
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
            ghostdag_store,
            relations_store,
            header_store,
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
        Ok(BlockDAG::new(8, dag_storage))
    }

    pub fn new_by_config(db_path: &Path) -> anyhow::Result<BlockDAG> {
        let config = FlexiDagStorageConfig::create_with_params(1, RocksdbConfig::default());
        let db = FlexiDagStorage::create_from_path(db_path, config)?;
        let dag = Self::new(8, db);
        Ok(dag)
    }

    pub fn has_dag_block(&self, hash: Hash) -> anyhow::Result<bool> {
        Ok(self.storage.header_store.has(hash)?)
    }

    pub fn init_with_genesis(&self, genesis: BlockHeader) -> anyhow::Result<()> {
        let genesis_id = genesis.id();
        let origin = genesis.parent_hash();

        let real_origin = Hash::sha3_256_of(&[origin, genesis_id].encode()?);
        println!("jacktest: real_origin: {:?}", real_origin);

        if self.storage.relations_store.has(real_origin)? {
            return Ok(());
        }
        inquirer::init(&mut self.storage.reachability_store.clone(), real_origin)?;

        self.storage
            .relations_store
            .insert(real_origin, BlockHashes::new(vec![]))?;
        // self.storage
        //     .relations_store
        //     .insert(origin, BlockHashes::new(vec![]))?;
        self.commit(genesis)?;
        self.save_dag_state(genesis_id, DagState {
            tips: vec![genesis_id],
        })?;
        Ok(())
    }
    pub fn ghostdata(&self, parents: &[HashValue]) -> anyhow::Result<GhostdagData> {
        self.ghostdag_manager.ghostdag(parents)
    }

    pub fn ghostdata_by_hash(&self, hash: HashValue) -> anyhow::Result<Option<Arc<GhostdagData>>> {
        match self.storage.ghost_dag_store.get_data(hash) {
            Result::Ok(value) => Ok(Some(value)),
            Err(StoreError::KeyNotFound(_)) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    fn process_key_already_error(&self, result: Result<(), StoreError>) -> Result<(), StoreError> {
        if let Err(StoreError::KeyAlreadyExists(_)) = result {
            Result::Ok(())
        } else {
            result
        }
    }

    pub fn commit(&self, header: BlockHeader) -> anyhow::Result<()> {
        // Generate ghostdag data
        println!("jacktest 1");
        let parents = header.parents();
        println!("jacktest 1.1: {:?}", parents);
        let ghostdata = match self.ghostdata_by_hash(header.id())? {
            None => {
                if header.is_dag_genesis() {
                    Arc::new(self.ghostdag_manager.genesis_ghostdag_data(&header))
                } else {
                    let ghostdata = self.ghostdag_manager.ghostdag(&parents)?;
                    Arc::new(ghostdata)
                }
            }
            Some(ghostdata) => ghostdata,
        };
        println!("jacktest 2.4: selected parent: {:?}", ghostdata.selected_parent);

        println!("jacktest 2");
        // Store ghostdata
        self.process_key_already_error(self.storage
            .ghost_dag_store
            .insert(header.id(), ghostdata.clone()))?;
        println!("jacktest 2.1");

        // Update reachability store
        let mut reachability_store = self.storage.reachability_store.clone();
        println!("jacktest 2.2: selected parent: {:?}", ghostdata.selected_parent);
        let mut merge_set = ghostdata
            .unordered_mergeset_without_selected_parent()
            .filter(|hash| self.storage.reachability_store.has(*hash).unwrap());
        println!("jacktest 2.3: selected parent: {:?}", ghostdata.selected_parent);
        inquirer::add_block(
            &mut reachability_store,
            header.id(),
            ghostdata.selected_parent,
            &mut merge_set,
        )?;
        println!("jacktest 3");
        // store relations
        if header.is_dag_genesis() {
            let origin = header.parent_hash();
            let real_origin = Hash::sha3_256_of(&[origin, header.id()].encode()?);
            self.storage
                .relations_store
                .insert(header.id(), BlockHashes::new(vec![real_origin]))?;
        } else {
            self.process_key_already_error(self.storage
                .relations_store
                .insert(header.id(), BlockHashes::new(parents)))?;
        }
        // Store header store
        self.process_key_already_error(self.storage
            .header_store
            .insert(header.id(), Arc::new(header), 0))?;
        println!("jacktest 4");
        Ok(())
    }

    pub fn get_parents(&self, hash: Hash) -> anyhow::Result<Vec<Hash>> {
        match self.storage.relations_store.get_parents(hash) {
            anyhow::Result::Ok(parents) => anyhow::Result::Ok((*parents).clone()),
            Err(error) => {
                println!("failed to get parents by hash: {}", error);
                bail!("failed to get parents by hash: {}", error);
            }
        }
    }

    pub fn get_children(&self, hash: Hash) -> anyhow::Result<Vec<Hash>> {
        match self.storage.relations_store.get_children(hash) {
            anyhow::Result::Ok(children) => anyhow::Result::Ok((*children).clone()),
            Err(error) => {
                println!("failed to get parents by hash: {}", error);
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
    use crate::consensusdb::{consenses_state::{DagState, DagStateReader, DagStateStore}, prelude::FlexiDagStorageConfig, schemadb::REACHABILITY_DATA_CF};
    use starcoin_config::RocksdbConfig;
    use starcoin_logger::prelude::{debug, info};
    use starcoin_storage::db_storage::DBStorage;
    use starcoin_types::block::{set_test_flexidag_fork_height, BlockHeader, BlockHeaderBuilder};
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
        let genesis = BlockHeader::dag_genesis_random()
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

    #[tokio::test]
    async fn test_with_spawn() {
        use starcoin_types::block::{BlockHeader, BlockHeaderBuilder};
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

    #[test]
    fn test_dag_genesis_fork() {
        // initialzie the dag firstly
        let dag = build_block_dag(3);

        let genesis = BlockHeader::dag_genesis_random()
            .as_builder()
            .with_difficulty(0.into())
            .build();
        dag.init_with_genesis(genesis.clone()).unwrap();

        // normally add the dag blocks
        let mut parents_hash = vec![genesis.id()];
        for _ in 0..10 {
            let header_builder = BlockHeaderBuilder::random();
            let header = header_builder
                .with_parents_hash(Some(parents_hash.clone()))
                .build();
            parents_hash = vec![header.id()];
            dag.commit(header.to_owned()).unwrap();
            let ghostdata = dag.ghostdata_by_hash(header.id()).unwrap().unwrap();
        }

        // fork, produce a new dag gensis
        let new_genesis = BlockHeader::dag_genesis_random()
            .as_builder()
            .with_difficulty(0.into())
            .build();
        dag.init_with_genesis(new_genesis.clone()).unwrap();

        // record the old dag chain
        let mut old_parents_hash = parents_hash.clone();
        // the new dag chain
        parents_hash = vec![new_genesis.id()];

        // add dag blocks in the old dag chain
        for _ in 0..10 {
            let header_builder = BlockHeaderBuilder::random();
            let header = header_builder
                .with_parents_hash(Some(old_parents_hash.clone()))
                .build();
            old_parents_hash = vec![header.id()];
            dag.commit(header.to_owned()).unwrap();
            let ghostdata = dag.ghostdata_by_hash(header.id()).unwrap().unwrap();
            println!("add a old header: {:?}, tips: {:?}", header, ghostdata);
        }

        // add dag blocks in the new dag chain
        for _ in 0..10 {
            let header_builder = BlockHeaderBuilder::random();
            let header = header_builder
                .with_parents_hash(Some(parents_hash.clone()))
                .build();
            parents_hash = vec![header.id()];
            dag.commit(header.to_owned()).unwrap();
            let ghostdata = dag.ghostdata_by_hash(header.id()).unwrap().unwrap();
            println!("add a forked header: {:?}, tips: {:?}", header, ghostdata);
        }

        let header_builder = BlockHeaderBuilder::random();
        parents_hash.append(&mut old_parents_hash);
        let header = header_builder.with_parents_hash(Some(parents_hash)).build();
        parents_hash = vec![header.id()];
        dag.commit(header.to_owned()).unwrap();
        let ghostdata = dag.ghostdata_by_hash(header.id()).unwrap().unwrap();
        println!("add a forked header: {:?}, tips: {:?}", header, ghostdata);
    }

    #[test]
    fn test_dag_tips_store() {
        let dag = BlockDAG::create_for_testing().unwrap();

        let state1 = DagState {
            tips: vec![Hash::random()],
        };
        let dag_gensis1 = Hash::random();
        dag.storage.state_store.insert(dag_gensis1, state1.clone()).expect("failed to store the dag state");

        let state2 = DagState {
            tips: vec![Hash::random()],
        };
        let dag_gensis2 = Hash::random();
        dag.storage.state_store.insert(dag_gensis2, state2.clone()).expect("failed to store the dag state");

        assert_eq!(dag.storage.state_store.get_state(dag_gensis1).expect("failed to get the dag state"), state1);
        assert_eq!(dag.storage.state_store.get_state(dag_gensis2).expect("failed to get the dag state"), state2);
    }

    #[test]
    fn test_dag_multiple_commits() {
        // initialzie the dag firstly
        let dag = BlockDAG::create_for_testing().unwrap();

        let genesis = BlockHeader::dag_genesis_random()
            .as_builder()
            .with_difficulty(0.into())
            .build();
        dag.init_with_genesis(genesis.clone()).unwrap();

        // normally add the dag blocks
        let mut headers = vec![];
        let mut parents_hash = vec![genesis.id()];
        let mut parent_hash = genesis.id();
        for _ in 0..100 {
            let header_builder = BlockHeaderBuilder::random();
            let header = header_builder
            .with_parent_hash(parent_hash)
                .with_parents_hash(Some(parents_hash.clone()))
                .build();
            parents_hash = vec![header.id()];
            parent_hash = header.id();
            headers.push(header.clone());
            dag.commit(header.to_owned()).unwrap();
            let ghostdata = dag.ghostdata_by_hash(header.id()).unwrap().unwrap();
        }

        for _ in 0..10 {
            for header in &headers {
                let _ = dag.commit(header.clone());
                let _ = dag.ghostdata_by_hash(header.id()).unwrap().unwrap();
            }
        }
    }

    #[test]
    fn test_reachability_store() -> anyhow::Result<()> {
        set_test_flexidag_fork_height(1);
        // initialzie the dag firstly
        let dag = BlockDAG::create_for_testing().unwrap();

        let origin = BlockHeaderBuilder::random().with_number(0).build();
        let genesis = BlockHeader::dag_genesis_random_with_parent(origin.clone());

        // let genesis = BlockHeader::dag_genesis_random()
        //     .as_builder()
        //     .with_difficulty(0.into())
        //     .build();
        dag.init_with_genesis(genesis.clone()).unwrap();

        // normally add the dag blocks
        let mut parents_hash = vec![genesis.id()];
        let mut parent_hash = genesis.id();
        // let genesis_other = BlockHeader::dag_genesis_random_with_parent(origin.clone());
        for i in 2..100 {
            // if i == 20 {
            //     dag.init_with_genesis(genesis_other.clone()).unwrap();
            // }
            // if i == 25 {
            //     parents_hash.append(&mut vec![genesis_other.id()]);
            // }
            let header_builder = BlockHeaderBuilder::random();
            let header = header_builder
            .with_parent_hash(parent_hash)
                .with_parents_hash(Some(parents_hash.clone()))
                .with_number(i)
                .build();
            // if i < 25 {
                parents_hash = vec![header.id()];
            // } else {
            //     parents_hash = vec![header.id(), genesis_other.id()];
            // }
            parent_hash = header.id();
            // headers.push(header.clone());
            println!("jacktest: before add a header: {:?}", header);
            let result = dag.commit(header.to_owned());
            println!("jacktest: again, result: {:?}", result);
            result?;
            let ghostdata = dag.ghostdata(&parents_hash).unwrap();
            println!("add a header: {:?}, tips: {:?}", header, ghostdata);
        }

        Ok(())
    }
}
