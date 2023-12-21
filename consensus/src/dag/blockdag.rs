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
use starcoin_config::{ChainNetworkID, NodeConfig, RocksdbConfig};
use starcoin_crypto::{HashValue as Hash, HashValue};
use starcoin_storage::Store;
use starcoin_types::block::{
    BlockHeader, BlockNumber, BARNARD_FLEXIDAG_FORK_HEIGHT, DEV_FLEXIDAG_FORK_HEIGHT,
    HALLEY_FLEXIDAG_FORK_HEIGHT, MAIN_FLEXIDAG_FORK_HEIGHT, PROXIMA_FLEXIDAG_FORK_HEIGHT,
    TEST_FLEXIDAG_FORK_HEIGHT,
};
use starcoin_types::dag_block::KTotalDifficulty;
use starcoin_types::startup_info::DagStartupInfo;
use starcoin_types::{
    blockhash::{BlockHashes, KType},
    consensus_header::ConsensusHeader,
};
use std::collections::BTreeSet;
use std::fmt;
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
    storage: FlexiDagStorage,
    ghostdag_manager: DbGhostdagManager,
    net: ChainNetworkID,
}

impl BlockDAG {
    pub fn new(k: KType, db: FlexiDagStorage, net: ChainNetworkID) -> Self {
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
            net,
        }
    }

    pub fn dag_fork_height_with_net(net: ChainNetworkID) -> BlockNumber {
        match net {
            ChainNetworkID::Builtin(network_id) => match network_id {
                starcoin_config::BuiltinNetworkID::Test => TEST_FLEXIDAG_FORK_HEIGHT,
                starcoin_config::BuiltinNetworkID::Dev => DEV_FLEXIDAG_FORK_HEIGHT,
                starcoin_config::BuiltinNetworkID::Halley => HALLEY_FLEXIDAG_FORK_HEIGHT,
                starcoin_config::BuiltinNetworkID::Proxima => PROXIMA_FLEXIDAG_FORK_HEIGHT,
                starcoin_config::BuiltinNetworkID::Barnard => BARNARD_FLEXIDAG_FORK_HEIGHT,
                starcoin_config::BuiltinNetworkID::Main => MAIN_FLEXIDAG_FORK_HEIGHT,
            },
            ChainNetworkID::Custom(_) => DEV_FLEXIDAG_FORK_HEIGHT,
        }
    }

    fn dag_fork_height(&self) -> BlockNumber {
        Self::dag_fork_height_with_net(self.net.clone())
    }

    pub fn try_init_with_storage(
        storage: Arc<dyn Store>,
        config: Arc<NodeConfig>,
    ) -> anyhow::Result<Option<Self>> {
        let startup_info = storage
            .get_startup_info()?
            .expect("startup info must exist");

        let block_header = storage
            .get_block_header_by_hash(startup_info.get_main().clone())?
            .expect("the genesis block in dag accumulator must none be none");

        let fork_height = Self::dag_fork_height_with_net(config.net().id().clone());

        if block_header.number() < fork_height {
            return Ok(None);
        } else if block_header.number() == fork_height {
            let dag = Self::new_by_config(
                config.data_dir().join("flexidag").as_path(),
                config.net().id().clone(),
            )?;
            dag.init_with_genesis(block_header)?;
            Ok(Some(dag))
        } else {
            Ok(Some(Self::new_by_config(
                config.data_dir().join("flexidag").as_path(),
                config.net().id().clone(),
            )?))
        }
    }

    pub fn new_by_config(db_path: &Path, net: ChainNetworkID) -> anyhow::Result<BlockDAG> {
        let config = FlexiDagStorageConfig::create_with_params(1, RocksdbConfig::default());
        let db = FlexiDagStorage::create_from_path(db_path, config)?;
        let dag = Self::new(16, db, net);
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

        let ghostdag_data = if header.number() == self.dag_fork_height() {
            self.ghostdag_manager.genesis_ghostdag_data(&header)
        } else {
            self.ghostdag_manager.ghostdag(parents_hash.as_slice())
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

    pub fn get_ghostdag_data_by_child(&self, hash: Hash) -> anyhow::Result<Arc<GhostdagData>> {
        let ghostdata = self.storage.ghost_dag_store.get_data(hash)?;
        return Ok(ghostdata);
    }
}

impl fmt::Debug for BlockDAG {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("print BlockDAG").finish()
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
        let dag = BlockDAG::new(k, db, ChainNetworkID::TEST);
        dag.init_with_genesis(genesis).unwrap();
        let mut block = BlockHeader::random();
        block.set_parents(vec![genesis_hash]);
        dag.commit(block).unwrap();
        let data = dag.ghostdag_manager.ghostdag(&vec![genesis_hash]);
        assert_eq!(data.selected_parent, genesis_hash);
        assert_eq!(data.mergeset_blues[0], genesis_hash);
    }
}
