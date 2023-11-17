use super::ghostdag::protocol::GhostdagManager;
use super::reachability::{inquirer, reachability_service::MTReachabilityService};
use super::types::ghostdata::GhostdagData;
use crate::consensusdb::prelude::StoreError;
use crate::consensusdb::schemadb::GhostdagStoreReader;
use crate::consensusdb::{
    prelude::FlexiDagStorage,
    schemadb::{
        DbGhostdagStore, DbHeadersStore, DbReachabilityStore, DbRelationsStore, GhostdagStore,
        HeaderStore, ReachabilityStoreReader, RelationsStore, RelationsStoreReader,
    },
};
use crate::FlexiDagStorageConfig;
use anyhow::{anyhow, bail, Ok};
use bcs_ext::BCSCodec;
use parking_lot::RwLock;
use starcoin_accumulator::accumulator_info::AccumulatorInfo;
use starcoin_accumulator::node::AccumulatorStoreType;
use starcoin_accumulator::{Accumulator, MerkleAccumulator};
use starcoin_config::{NodeConfig, RocksdbConfig};
use starcoin_crypto::HashValue as Hash;
use starcoin_storage::flexi_dag::SyncFlexiDagSnapshotHasher;
use starcoin_storage::storage::CodecKVStore;
use starcoin_storage::{BlockStore, Storage, Store, SyncFlexiDagStore};
use starcoin_types::block::BlockNumber;
use starcoin_types::dag_block::KTotalDifficulty;
use starcoin_types::startup_info;
use starcoin_types::{
    blockhash::{BlockHashes, KType},
    consensus_header::ConsensusHeader,
};
use std::collections::{HashSet, BTreeSet};
use std::collections::{BinaryHeap, HashMap};
use std::path::Path;
use std::sync::{Arc, Mutex};

pub type DbGhostdagManager = GhostdagManager<
    DbGhostdagStore,
    DbRelationsStore,
    MTReachabilityService<DbReachabilityStore>,
    DbHeadersStore,
>;

#[derive(Clone)]
pub struct BlockDAG {
    storage: FlexiDagStorage,
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

        let mut dag = Self {
            ghostdag_manager,
            storage: db,
        };
        dag
    }

    pub fn calculate_dag_accumulator_key(snapshot: &SyncFlexiDagSnapshotHasher) -> anyhow::Result<Hash> {
        Ok(Hash::sha3_256_of(&snapshot.encode().expect(
            "encoding the sorted relatship set must be successful",
        )))
    }

    pub fn try_init_with_storage(
        storage: Arc<dyn Store>,
        config: Arc<NodeConfig>,
    ) -> anyhow::Result<(Option<Self>, Option<MerkleAccumulator>)> {
        let startup_info = storage
            .get_startup_info()?
            .expect("startup info must exist");
        if let Some(key) = startup_info.get_dag_main() {
            let accumulator_info = storage
                .get_dag_accumulator_info()?
                .expect("dag accumulator info should exist");
            assert!(
                accumulator_info.get_num_leaves() > 0,
                "the number of dag accumulator leaf must be greater than 0"
            );
            let dag_accumulator = MerkleAccumulator::new_with_info(
                accumulator_info,
                storage.get_accumulator_store(AccumulatorStoreType::SyncDag),
            );
 
            Ok((
                Some(Self::new_by_config(
                    config.data_dir().join("flexidag").as_path(),
                )?),
                Some(dag_accumulator),
            ))
        } else {
            let block_header = storage
                .get_block_header_by_hash(startup_info.get_main().clone())?
                .expect("the genesis block in dag accumulator must none be none");
            let fork_height = storage.dag_fork_height(config.net().id().clone());
            if block_header.number() < fork_height {
                Ok((None, None))
            } else if block_header.number() == fork_height {
                let dag_accumulator = MerkleAccumulator::new_with_info(
                    AccumulatorInfo::default(),
                    storage.get_accumulator_store(AccumulatorStoreType::SyncDag),
                );


                let mut k_total_difficulties = BTreeSet::new();
                k_total_difficulties.insert(KTotalDifficulty {
                    head_block_id: block_header.id(),
                    total_difficulty: storage
                        .get_block_info(block_header.id())?
                        .expect("block info must exist")
                        .get_total_difficulty(),
                });
                let snapshot_hasher = SyncFlexiDagSnapshotHasher {
                    child_hashes: vec![block_header.id()],
                    head_block_id: block_header.id(),
                    k_total_difficulties,
                }; 
                let key = Self::calculate_dag_accumulator_key(&snapshot_hasher)?;
                dag_accumulator.append(&[key])?;
                storage.get_accumulator_snapshot_storage().put(
                    key,
                    snapshot_hasher.to_snapshot(dag_accumulator.get_info()),
                )?;
                dag_accumulator.flush()?;
                Ok((
                    Some(Self::new_by_config(
                        config.data_dir().join("flexidag").as_path(),
                    )?),
                    Some(dag_accumulator),
                ))
            } else {
                bail!("failed to init dag")
            }
        }
    }

    pub fn new_by_config(db_path: &Path) -> anyhow::Result<BlockDAG> {
        let config = FlexiDagStorageConfig::create_with_params(1, RocksdbConfig::default());
        let db = FlexiDagStorage::create_from_path(db_path, config)?;
        let dag = Self::new(16, db);
        Ok(dag)
    }
    pub fn init_with_genesis(&self, genesis: BlockHeader) -> anyhow::Result<()> {
        let origin = genesis.parent_hash();
        if self.storage.relations_store.has(origin)? {
            return Err(anyhow!("Already init with genesis"));
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

    pub fn commit(&self, header: BlockHeader) -> anyhow::Result<()> {
        // Generate ghostdag data
        let parents_hash = header.parents();

        let ghostdag_data = if !header.is_dag_genesis() {
            self.ghostdag_manager.ghostdag(parents_hash.as_slice())
        } else {
            self.ghostdag_manager.genesis_ghostdag_data(&header)
        };
        // Store ghostdata
        self.storage
            .ghost_dag_store
            .insert(header.id(), Arc::new(ghostdag_data.clone()))?;

        // Update reachability store
        let mut reachability_store = self.storage.reachability_store.clone();
        let mut merge_set = ghostdag_data
            .unordered_mergeset_without_selected_parent()
            .filter(|hash| self.storage.reachability_store.has(*hash).unwrap());

        inquirer::add_block(
            &mut reachability_store,
            header.id(),
            ghostdag_data.selected_parent,
            &mut merge_set,
        )?;

        // store relations
        self.storage
            .relations_store
            .insert(header.id(), BlockHashes::new(parents_hash.to_vec()))?;
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

    // for testing
    pub fn push_parent_children(
        &mut self,
        child: Hash,
        parents: Arc<Vec<Hash>>,
    ) -> Result<(), StoreError> {
        self.storage.relations_store.insert(child, parents)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FlexiDagStorageConfig;
    use starcoin_config::RocksdbConfig;
    use starcoin_types::block::BlockHeader;
    use std::{env, fs};

    #[test]
    fn base_test() {
        let genesis = BlockHeader::dag_genesis_random();
        let genesis_hash = genesis.hash();
        let k = 16;
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
        let mut dag = BlockDAG::new(k, db);
        dag.init_with_genesis(genesis).unwrap();
        let mut block = BlockHeader::random();
        block.set_parents(vec![genesis_hash]);
        dag.commit(block).unwrap();
    }
}
