use super::reachability::{inquirer, reachability_service::MTReachabilityService};
use super::types::ghostdata::GhostdagData;
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
use anyhow::{bail, Ok};
use bcs_ext::BCSCodec;
use parking_lot::RwLock;
use starcoin_config::{temp_dir, RocksdbConfig};
use starcoin_crypto::{HashValue as Hash, HashValue};
use starcoin_logger::prelude::info;
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

    pub fn init_with_genesis(&mut self, genesis: BlockHeader) -> anyhow::Result<()> {
        let genesis_id = genesis.id();
        let origin = genesis.parent_hash();

        let real_origin = Hash::sha3_256_of(&[origin, genesis_id].encode()?);

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
        self.commit(genesis, origin)?;
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

    pub fn commit(&mut self, header: BlockHeader, origin: HashValue) -> anyhow::Result<()> {
        // Generate ghostdag data
        let parents = header.parents();
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
        // Store ghostdata
        process_key_already_error(self.storage
            .ghost_dag_store
            .insert(header.id(), ghostdata.clone()))?;

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
                let _future_covering_set = reachability_store.get_future_covering_set(header.id())?;
                info!("the key {:?} was already processed, original error message: {:?}", header.id(), reachability::ReachabilityError::DataInconsistency);
            }
            Err(reachability::ReachabilityError::StoreError(StoreError::KeyNotFound(msg))) => {
                if msg == REINDEX_ROOT_KEY.to_string() {
                    info!("the key {:?} was already processed, original error message: {:?}", header.id(), reachability::ReachabilityError::StoreError(StoreError::KeyNotFound(REINDEX_ROOT_KEY.to_string())));
                    info!("now set the reindex key to origin: {:?}", origin);
                    self.storage.reachability_store.set_reindex_root(origin)?;
                    bail!("failed to add a block when committing, e: {:?}", reachability::ReachabilityError::StoreError(StoreError::KeyNotFound(msg)));

                } else {
                    bail!("failed to add a block when committing, e: {:?}", reachability::ReachabilityError::StoreError(StoreError::KeyNotFound(msg)));
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
            process_key_already_error(self.storage
                .relations_store
                .insert(header.id(), BlockHashes::new(vec![real_origin])))?;
        } else {
            process_key_already_error(self.storage
                .relations_store
                .insert(header.id(), BlockHashes::new(parents)))?;
        }
        // Store header store
        process_key_already_error(self.storage
            .header_store
            .insert(header.id(), Arc::new(header), 0))?;
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

