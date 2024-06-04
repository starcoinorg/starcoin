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
use anyhow::{anyhow, bail, Ok};
use bcs_ext::BCSCodec;
use starcoin_config::{temp_dir, RocksdbConfig};
use starcoin_crypto::{HashValue as Hash, HashValue};
use starcoin_logger::prelude::info;
use starcoin_types::block::BlockHeader;
use starcoin_types::{
    blockhash::{BlockHashes, KType},
    consensus_header::ConsensusHeader,
};
use std::ops::DerefMut;
use std::path::Path;
use std::sync::Arc;

pub const DEFAULT_GHOSTDAG_K: KType = 8u16;

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
        let reachability_service = MTReachabilityService::new(reachability_store);
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
        Ok(BlockDAG::new(DEFAULT_GHOSTDAG_K, dag_storage))
    }

    pub fn new_by_config(db_path: &Path) -> anyhow::Result<BlockDAG> {
        let config = FlexiDagStorageConfig::create_with_params(1, RocksdbConfig::default());
        let db = FlexiDagStorage::create_from_path(db_path, config)?;
        let dag = Self::new(DEFAULT_GHOSTDAG_K, db);
        Ok(dag)
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

        if self.storage.relations_store.read().has(real_origin)? {
            return Ok(real_origin);
        }
        inquirer::init(
            self.storage.reachability_store.write().deref_mut(),
            real_origin,
        )?;

        self.storage
            .relations_store
            .write()
            .insert(real_origin, BlockHashes::new(vec![]))?;

        self.commit(genesis, real_origin)?;
        self.save_dag_state(
            genesis_id,
            DagState {
                tips: vec![genesis_id],
            },
        )?;
        Ok(real_origin)
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

    pub fn set_reindex_root(&mut self, hash: HashValue) -> anyhow::Result<()> {
        self.storage
            .reachability_store
            .write()
            .set_reindex_root(hash)?;
        Ok(())
    }

    pub fn commit(&mut self, header: BlockHeader, origin: HashValue) -> anyhow::Result<()> {
        // Generate ghostdag data
        let parents = header.parents();
        let ghostdata = match self.ghostdata_by_hash(header.id())? {
            None => {
                // It must be the dag genesis if header is a format for a single chain
                if header.is_single() {
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
        let reachability_store = self.storage.reachability_store.clone();
        let mut merge_set = ghostdata
            .unordered_mergeset_without_selected_parent()
            .filter(|hash| self.storage.reachability_store.read().has(*hash).unwrap())
            .collect::<Vec<_>>()
            .into_iter();
        let add_block_result = {
            let mut reachability_writer = reachability_store.write();
            inquirer::add_block(
                reachability_writer.deref_mut(),
                header.id(),
                ghostdata.selected_parent,
                &mut merge_set,
            )
        };
        match add_block_result {
            Result::Ok(_) => (),
            Err(reachability::ReachabilityError::DataInconsistency) => {
                let _future_covering_set = reachability_store
                    .read()
                    .get_future_covering_set(header.id())?;
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
            Err(reachability::ReachabilityError::StoreError(StoreError::InvalidInterval(_, _))) => {
                self.set_reindex_root(origin)?;
                bail!("failed to add a block when committing for invalid interval",);
            }
            Err(e) => {
                bail!("failed to add a block when committing, e: {:?}", e);
            }
        }

        // store relations
        // It must be the dag genesis if header is a format for a single chain
        if header.is_single() {
            let origin = header.parent_hash();
            let real_origin = Hash::sha3_256_of(&[origin, header.id()].encode()?);
            process_key_already_error(
                self.storage
                    .relations_store
                    .write()
                    .insert(header.id(), BlockHashes::new(vec![real_origin])),
            )?;
        } else {
            process_key_already_error(
                self.storage
                    .relations_store
                    .write()
                    .insert(header.id(), BlockHashes::new(parents)),
            )?;
        }
        // Store header store
        process_key_already_error(self.storage.header_store.insert(
            header.id(),
            Arc::new(header),
            1,
        ))?;
        Ok(())
    }

    pub fn get_parents(&self, hash: Hash) -> anyhow::Result<Vec<Hash>> {
        match self.storage.relations_store.write().get_parents(hash) {
            anyhow::Result::Ok(parents) => anyhow::Result::Ok((*parents).clone()),
            Err(error) => {
                bail!("failed to get parents by hash: {}", error);
            }
        }
    }

    pub fn get_children(&self, hash: Hash) -> anyhow::Result<Vec<Hash>> {
        match self.storage.relations_store.read().get_children(hash) {
            anyhow::Result::Ok(children) => anyhow::Result::Ok((*children).clone()),
            Err(error) => {
                bail!("failed to get parents by hash: {}", error);
            }
        }
    }

    pub fn get_dag_state(&self, hash: Hash) -> anyhow::Result<DagState> {
        Ok(self.storage.state_store.read().get_state(hash)?)
    }

    pub fn save_dag_state(&self, hash: Hash, state: DagState) -> anyhow::Result<()> {
        self.storage.state_store.write().insert(hash, state)?;
        Ok(())
    }

    pub fn load_dag_genesis(&self) -> anyhow::Result<Option<HashValue>> {
        let state = self
            .storage
            .state_store
            .read()
            .iter()?
            .flatten()
            .take(2)
            .map(|(hash, _)| hash)
            .collect::<Vec<_>>();

        match state.len() {
            0 => Ok(None),
            1 => Ok(Some(*state.first().unwrap())),
            _ => Err(anyhow!("more thane one dag genesis found")),
        }
    }
}
