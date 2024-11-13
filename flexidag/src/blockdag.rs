use super::reachability::{inquirer, reachability_service::MTReachabilityService};
use super::types::ghostdata::GhostdagData;
use crate::consensusdb::consenses_state::{
    DagState, DagStateReader, DagStateStore, ReachabilityView,
};
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
use crate::prune::pruning_point_manager::PruningPointManagerT;
use crate::{process_key_already_error, reachability};
use anyhow::{bail, ensure, Ok};
use starcoin_config::temp_dir;
use starcoin_crypto::{HashValue as Hash, HashValue};
use starcoin_logger::prelude::{debug, info, warn};
use starcoin_types::block::BlockHeader;
use starcoin_types::{
    blockhash::{BlockHashes, KType},
    consensus_header::ConsensusHeader,
};
use std::collections::HashSet;
use std::ops::DerefMut;
use std::sync::Arc;

pub const DEFAULT_GHOSTDAG_K: KType = 8u16;

pub type DbGhostdagManager = GhostdagManager<
    DbGhostdagStore,
    DbRelationsStore,
    MTReachabilityService<DbReachabilityStore>,
    DbHeadersStore,
>;

pub type PruningPointManager = PruningPointManagerT<DbReachabilityStore>;

pub struct MineNewDagBlockInfo {
    pub tips: Vec<HashValue>,
    pub blue_blocks: Vec<HashValue>,
    pub pruning_point: HashValue,
}

#[derive(Clone)]
pub struct BlockDAG {
    pub storage: FlexiDagStorage,
    ghostdag_manager: DbGhostdagManager,
    pruning_point_manager: PruningPointManager,
}

impl BlockDAG {
    pub fn create_blockdag(dag_storage: FlexiDagStorage) -> Self {
        Self::new(DEFAULT_GHOSTDAG_K, dag_storage)
    }

    pub fn new(k: KType, db: FlexiDagStorage) -> Self {
        let ghostdag_store = db.ghost_dag_store.clone();
        let header_store = db.header_store.clone();
        let relations_store = db.relations_store.clone();
        let reachability_store = db.reachability_store.clone();
        let reachability_service = MTReachabilityService::new(reachability_store);
        let ghostdag_manager = DbGhostdagManager::new(
            k,
            ghostdag_store.clone(),
            relations_store,
            header_store,
            reachability_service.clone(),
        );
        let pruning_point_manager = PruningPointManager::new(reachability_service, ghostdag_store);

        Self {
            ghostdag_manager,
            storage: db,
            pruning_point_manager,
        }
    }

    pub fn create_for_testing() -> anyhow::Result<Self> {
        let config = FlexiDagStorageConfig {
            cache_size: 1024,
            ..Default::default()
        };
        let dag_storage = FlexiDagStorage::create_from_path(temp_dir(), config)?;
        Ok(Self::create_blockdag(dag_storage))
    }

    pub fn create_for_testing_with_parameters(k: KType) -> anyhow::Result<Self> {
        let dag_storage =
            FlexiDagStorage::create_from_path(temp_dir(), FlexiDagStorageConfig::default())?;
        Ok(Self::new(k, dag_storage))
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

        inquirer::init(self.storage.reachability_store.write().deref_mut(), origin)?;

        self.storage
            .relations_store
            .write()
            .insert(origin, BlockHashes::new(vec![]))?;

        self.commit(genesis, origin)?;
        self.save_dag_state(
            genesis_id,
            DagState {
                tips: vec![genesis_id],
            },
        )?;
        Ok(origin)
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

    pub fn pruning_point_manager(&self) -> PruningPointManager {
        self.pruning_point_manager.clone()
    }

    pub fn set_reindex_root(&mut self, hash: HashValue) -> anyhow::Result<()> {
        self.storage
            .reachability_store
            .write()
            .set_reindex_root(hash)?;
        Ok(())
    }

    pub fn commit_trusted_block(
        &mut self,
        header: BlockHeader,
        origin: HashValue,
        trusted_ghostdata: Arc<GhostdagData>,
    ) -> anyhow::Result<()> {
        info!(
            "start to commit header: {:?}, number: {:?}",
            header.id(),
            header.number()
        );
        // Generate ghostdag data
        let parents = header.parents();

        debug!(
            "start to get the ghost data from block: {:?}, number: {:?}",
            header.id(),
            header.number()
        );
        let ghostdata = match self.ghostdata_by_hash(header.id())? {
            None => {
                // It must be the dag genesis if header is a format for a single chain
                if header.is_genesis() {
                    Arc::new(self.ghostdag_manager.genesis_ghostdag_data(&header))
                } else {
                    self.storage
                        .ghost_dag_store
                        .insert(header.id(), trusted_ghostdata.clone())?;
                    trusted_ghostdata
                }
            }
            Some(ghostdata) => {
                ensure!(
                    ghostdata.blue_score == trusted_ghostdata.blue_score,
                    "blue score is not same"
                );
                ensure!(
                    ghostdata.blue_work == trusted_ghostdata.blue_work,
                    "blue work is not same"
                );
                ensure!(
                    ghostdata.mergeset_blues.len() == trusted_ghostdata.mergeset_blues.len(),
                    "blue len is not same"
                );
                ensure!(
                    ghostdata
                        .mergeset_blues
                        .iter()
                        .cloned()
                        .collect::<HashSet<_>>()
                        == trusted_ghostdata
                            .mergeset_blues
                            .iter()
                            .cloned()
                            .collect::<HashSet<_>>(),
                    "blue values are not same"
                );
                trusted_ghostdata
            }
        };

        // Store ghostdata
        process_key_already_error(
            self.storage
                .ghost_dag_store
                .insert(header.id(), ghostdata.clone()),
        )?;

        // Update reachability store
        debug!(
            "start to update reachability data for block: {:?}, number: {:?}",
            header.id(),
            header.number()
        );
        let reachability_store = self.storage.reachability_store.clone();

        let mut merge_set = self
            .ghost_dag_manager()
            .unordered_mergeset_without_selected_parent(
                ghostdata.selected_parent,
                &header.parents(),
            )
            .into_iter();
        // let mut merge_set = ghostdata
        //     .unordered_mergeset_without_selected_parent()
        //     .filter(|hash| self.storage.reachability_store.read().has(*hash).unwrap())
        //     .chain(
        //         header
        //             .parents_hash()
        //             .into_iter()
        //             .filter(|parent_id| *parent_id != ghostdata.selected_parent),
        //     )
        //     .collect::<HashSet<_>>()
        //     .into_iter();
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
        process_key_already_error(
            self.storage
                .relations_store
                .write()
                .insert(header.id(), BlockHashes::new(parents)),
        )?;
        // Store header store
        process_key_already_error(self.storage.header_store.insert(
            header.id(),
            Arc::new(header),
            1,
        ))?;
        Ok(())
    }

    pub fn commit(&mut self, header: BlockHeader, origin: HashValue) -> anyhow::Result<()> {
        info!(
            "start to commit header: {:?}, number: {:?}",
            header.id(),
            header.number()
        );
        // Generate ghostdag data
        let parents = header.parents();

        debug!(
            "start to get the ghost data from block: {:?}, number: {:?}",
            header.id(),
            header.number()
        );
        let ghostdata = match self.ghostdata_by_hash(header.id())? {
            None => {
                // It must be the dag genesis if header is a format for a single chain
                if header.is_genesis() {
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
        debug!(
            "start to update reachability data for block: {:?}, number: {:?}",
            header.id(),
            header.number()
        );
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

        process_key_already_error(
            self.storage
                .relations_store
                .write()
                .insert(header.id(), BlockHashes::new(parents)),
        )?;
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
        Ok(self.storage.state_store.read().get_state_by_hash(hash)?)
    }

    pub fn save_dag_state_directly(&self, hash: Hash, state: DagState) -> anyhow::Result<()> {
        self.storage.state_store.write().insert(hash, state)?;
        anyhow::Ok(())
    }

    pub fn save_dag_state(&self, hash: Hash, state: DagState) -> anyhow::Result<()> {
        let writer = self.storage.state_store.write();
        match writer.get_state_by_hash(hash) {
            anyhow::Result::Ok(dag_state) => {
                // remove the ancestor tips
                let left_tips = dag_state.tips.into_iter().filter(|tip| {
                    !state.tips.iter().any(|new_tip| {
                        self.ghost_dag_manager().check_ancestor_of(*tip, vec![*new_tip]).unwrap_or_else(|e| {
                            warn!("failed to check ancestor of tip: {:?}, new_tip: {:?}, error: {:?}", tip, new_tip, e);
                            false
                        })
                    })
                });
                let merged_tips = left_tips
                    .chain(state.tips.clone())
                    .collect::<HashSet<_>>()
                    .into_iter()
                    .collect::<Vec<_>>();
                writer.insert(hash, DagState { tips: merged_tips })?;
            }
            Err(_) => {
                writer.insert(hash, state)?;
            }
        }

        drop(writer);

        Ok(())
    }

    pub fn ghost_dag_manager(&self) -> DbGhostdagManager {
        self.ghostdag_manager.clone()
    }

    pub fn calc_mergeset_and_tips(
        &self,
        previous_pruning_point: HashValue,
        previous_ghostdata: &GhostdagData,
        pruning_depth: u64,
        pruning_finality: u64,
    ) -> anyhow::Result<MineNewDagBlockInfo> {
        info!("start to calculate the mergeset and tips, previous pruning point: {:?}, previous ghostdata: {:?}", previous_pruning_point, previous_ghostdata);
        let dag_state = self.get_dag_state(previous_pruning_point)?;
        let next_ghostdata = self.ghostdata(&dag_state.tips)?;
        info!(
            "start to calculate the mergeset and tips for tips: {:?}, and last pruning point: {:?} and next ghostdata's selected parents: {:?} and blues set are {:?}",
            dag_state.tips, previous_pruning_point, next_ghostdata.selected_parent, next_ghostdata.mergeset_blues,
        );
        let next_pruning_point = self.pruning_point_manager().next_pruning_point(
            previous_pruning_point,
            previous_ghostdata,
            &next_ghostdata,
            pruning_depth,
            pruning_finality,
        )?;
        info!(
            "the next pruning point is: {:?}, and the previous pruning point is: {:?}",
            next_pruning_point, previous_pruning_point
        );
        if next_pruning_point == Hash::zero() || next_pruning_point == previous_pruning_point {
            anyhow::Ok(MineNewDagBlockInfo {
                tips: dag_state.tips,
                blue_blocks: (*next_ghostdata.mergeset_blues).clone(),
                pruning_point: next_pruning_point,
            })
        } else {
            let pruned_tips = self.pruning_point_manager().prune(
                &dag_state,
                previous_pruning_point,
                next_pruning_point,
            )?;
            let mergeset_blues = (*self
                .ghost_dag_manager()
                .ghostdag(&pruned_tips)?
                .mergeset_blues)
                .clone();
            info!(
                "previous tips are: {:?}, the pruned tips are: {:?}, the mergeset blues are: {:?}, the next pruning point is: {:?}",
                dag_state.tips,
                pruned_tips, mergeset_blues, next_pruning_point
            );
            anyhow::Ok(MineNewDagBlockInfo {
                tips: pruned_tips,
                blue_blocks: mergeset_blues,
                pruning_point: next_pruning_point,
            })
        }
    }

    pub fn verify_pruning_point(
        &self,
        previous_pruning_point: HashValue,
        previous_ghostdata: &GhostdagData,
        next_pruning_point: HashValue,
        next_ghostdata: &GhostdagData,
        pruning_depth: u64,
        pruning_finality: u64,
    ) -> anyhow::Result<()> {
        let inside_next_pruning_point = self.pruning_point_manager().next_pruning_point(
            previous_pruning_point,
            previous_ghostdata,
            next_ghostdata,
            pruning_depth,
            pruning_finality,
        )?;

        if next_pruning_point != inside_next_pruning_point {
            bail!("pruning point is not correct, the local next pruning point is {}, but the block header pruning point is {}", next_pruning_point, inside_next_pruning_point);
        }
        anyhow::Ok(())
    }

    pub fn reachability_store(
        &self,
    ) -> Arc<parking_lot::lock_api::RwLock<parking_lot::RawRwLock, DbReachabilityStore>> {
        self.storage.reachability_store.clone()
    }

    pub fn reachability_service(&self) -> MTReachabilityService<DbReachabilityStore> {
        self.pruning_point_manager().reachability_service()
    }

    pub fn verify_and_ghostdata(
        &self,
        blue_blocks: &[BlockHeader],
        header: &BlockHeader,
    ) -> Result<GhostdagData, anyhow::Error> {
        self.ghost_dag_manager()
            .verify_and_ghostdata(blue_blocks, header)
    }
    pub fn check_upgrade(&self, main: &BlockHeader, genesis_id: HashValue) -> anyhow::Result<()> {
        // set the state with key 0
        if main.version() == 0 || main.version() == 1 {
            let result_dag_state = self
                .storage
                .state_store
                .read()
                .get_state_by_hash(genesis_id);
            match result_dag_state {
                anyhow::Result::Ok(_dag_state) => (),
                Err(_) => {
                    let result_dag_state = self
                        .storage
                        .state_store
                        .read()
                        .get_state_by_hash(HashValue::zero());

                    match result_dag_state {
                        anyhow::Result::Ok(dag_state) => {
                            self.storage
                                .state_store
                                .write()
                                .insert(genesis_id, dag_state)?;
                        }
                        Err(_) => {
                            let dag_state = self
                                .storage
                                .state_store
                                .read()
                                .get_state_by_hash(main.id())?;
                            self.storage
                                .state_store
                                .write()
                                .insert(HashValue::zero(), dag_state.clone())?;
                            self.storage
                                .state_store
                                .write()
                                .insert(genesis_id, dag_state)?;
                        }
                    }
                }
            }
        }

        anyhow::Ok(())
    }

    pub fn is_ancestor_of(
        &self,
        ancestor: Hash,
        descendants: Vec<Hash>,
    ) -> anyhow::Result<ReachabilityView> {
        let de = descendants
            .into_iter()
            .filter(|descendant| {
                self.check_ancestor_of(ancestor, vec![*descendant])
                    .unwrap_or(false)
            })
            .collect::<Vec<_>>();
        anyhow::Ok(ReachabilityView {
            ancestor,
            descendants: de,
        })
    }
}
