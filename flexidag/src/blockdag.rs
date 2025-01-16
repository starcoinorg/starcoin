use super::reachability::{inquirer, reachability_service::MTReachabilityService};
use super::types::ghostdata::GhostdagData;
use crate::consensusdb::consenses_state::{
    DagState, DagStateReader, DagStateStore, ReachabilityView,
};
use crate::consensusdb::prelude::{FlexiDagStorageConfig, StoreError};
use crate::consensusdb::schemadb::{
    GhostdagStoreReader, ReachabilityStore, StagingReachabilityStore,
};
use crate::consensusdb::{
    prelude::FlexiDagStorage,
    schemadb::{
        DbGhostdagStore, DbHeadersStore, DbReachabilityStore, DbRelationsStore, GhostdagStore,
        HeaderStore, ReachabilityStoreReader, RelationsStore, RelationsStoreReader,
    },
};
use crate::ghostdag::protocol::GhostdagManager;
use crate::process_key_already_error;
use crate::prune::pruning_point_manager::PruningPointManagerT;
use crate::reachability::ReachabilityError;
use anyhow::{bail, ensure, Ok};
use itertools::Itertools;
use parking_lot::Mutex;
use rocksdb::WriteBatch;
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
    commit_lock: Arc<Mutex<FlexiDagStorage>>,
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
        let commit_lock = Arc::new(Mutex::new(db.clone()));
        Self {
            ghostdag_manager,
            storage: db,
            pruning_point_manager,
            commit_lock,
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

    pub fn has_block_connected(&self, block_header: &BlockHeader) -> anyhow::Result<bool> {
        match self.storage.ghost_dag_store.has(block_header.id()) {
            std::result::Result::Ok(true) => (),
            std::result::Result::Ok(false) => {
                warn!("failed to get ghostdata by hash, the block should be re-executed",);
                return anyhow::Result::Ok(false);
            }
            Err(e) => {
                warn!(
                    "failed to get ghostdata by hash: {:?}, the block should be re-executed",
                    e
                );
                return anyhow::Result::Ok(false);
            }
        };

        match self.storage.header_store.has(block_header.id()) {
            std::result::Result::Ok(true) => (),
            std::result::Result::Ok(false) => {
                warn!("failed to get header by hash, the block should be re-executed",);
                return anyhow::Result::Ok(false);
            }
            Err(e) => {
                warn!(
                    "failed to get header by hash: {:?}, the block should be re-executed",
                    e
                );
                return anyhow::Result::Ok(false);
            }
        };

        let parents = match self
            .storage
            .relations_store
            .read()
            .get_parents(block_header.id())
        {
            std::result::Result::Ok(parents) => parents,
            Err(e) => {
                warn!(
                    "failed to get parents by hash: {:?}, the block should be re-executed",
                    e
                );
                return anyhow::Result::Ok(false);
            }
        };

        if !parents.iter().all(|parent| {
            let children = match self.storage.relations_store.read().get_children(*parent) {
                std::result::Result::Ok(children) => children,
                Err(e) => {
                    warn!("failed to get children by hash: {:?}, the block should be re-executed", e);
                    return false;
                }
            };

            if !children.contains(&block_header.id()) {
                warn!("the parent: {:?} does not have the child: {:?}", parent, block_header.id());
                return false;
            }

            match inquirer::is_dag_ancestor_of(&*self.storage.reachability_store.read(), *parent, block_header.id()) {
                std::result::Result::Ok(pass) => {
                    if !pass {
                        warn!("failed to check ancestor, the block: {:?} is not the descendant of its parent: {:?}, the block should be re-executed", block_header.id(), *parent);
                        return false;
                    }
                    true
                }
                Err(e) => {
                    warn!("failed to check ancestor, the block: {:?} is not the descendant of its parent: {:?}, the block should be re-executed, error: {:?}", block_header.id(), *parent, e);
                    false
                }
            }
        }) {
            return anyhow::Result::Ok(false);
        }

        if block_header.pruning_point() == HashValue::zero() {
            return anyhow::Result::Ok(true);
        } else {
            match inquirer::is_dag_ancestor_of(
                &*self.storage.reachability_store.read(),
                block_header.pruning_point(),
                block_header.id(),
            ) {
                std::result::Result::Ok(pass) => {
                    if !pass {
                        warn!("failed to check ancestor, the block: {:?} is not the descendant of the pruning: {:?}", block_header.id(), block_header.pruning_point());
                        return anyhow::Result::Ok(false);
                    }
                }
                Err(e) => {
                    warn!("failed to check ancestor, the block: {:?} is not the descendant of the pruning: {:?}, error: {:?}", block_header.id(), block_header.pruning_point(), e);
                    return anyhow::Result::Ok(false);
                }
            }
        }

        anyhow::Result::Ok(true)
    }

    pub fn check_ancestor_of(&self, ancestor: Hash, descendant: Hash) -> anyhow::Result<bool> {
        inquirer::is_dag_ancestor_of(
            &*self.storage.reachability_store.read(),
            ancestor,
            descendant,
        )
        .map_err(|e| e.into())
    }

    pub fn init_with_genesis(&mut self, genesis: BlockHeader) -> anyhow::Result<HashValue> {
        let genesis_id = genesis.id();
        let origin = genesis.parent_hash();

        inquirer::init(self.storage.reachability_store.write().deref_mut(), origin)?;

        self.storage
            .relations_store
            .write()
            .insert(origin, BlockHashes::new(vec![]))?;

        self.commit(genesis)?;
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

        if header.pruning_point() == HashValue::zero() {
            info!(
                "try to hint virtual selected parent, root index: {:?}",
                self.storage.reachability_store.read().get_reindex_root()
            );
            let _ = inquirer::hint_virtual_selected_parent(
                self.storage.reachability_store.write().deref_mut(),
                header.parent_hash(),
            );
            info!(
                "after hint virtual selected parent, root index: {:?}",
                self.storage.reachability_store.read().get_reindex_root()
            );
        } else if self.storage.reachability_store.read().get_reindex_root()?
            != header.pruning_point()
            && self
                .storage
                .reachability_store
                .read()
                .has(header.pruning_point())?
        {
            info!(
                "try to hint virtual selected parent, root index: {:?}",
                self.storage.reachability_store.read().get_reindex_root()
            );
            let hint_result = inquirer::hint_virtual_selected_parent(
                self.storage.reachability_store.write().deref_mut(),
                header.pruning_point(),
            );
            info!(
                "after hint virtual selected parent, root index: {:?}, hint result: {:?}",
                self.storage.reachability_store.read().get_reindex_root(),
                hint_result
            );
        }

        // Create a DB batch writer
        let mut batch = WriteBatch::default();

        info!("start to commit via batch, header id: {:?}", header.id());
        let lock_guard = self.commit_lock.lock();

        // lock the dag data to write in batch
        // the cache will be written at the same time
        // when the batch is written before flush to the disk and
        // if the writing process abort the starcoin process will/should restart.
        let mut stage = StagingReachabilityStore::new(
            self.storage.db.clone(),
            self.storage.reachability_store.upgradable_read(),
        );

        // Store ghostdata
        process_key_already_error(self.storage.ghost_dag_store.insert_batch(
            &mut batch,
            header.id(),
            ghostdata.clone(),
        ))
        .expect("failed to ghostdata in batch");

        // Update reachability store
        debug!(
            "start to update reachability data for block: {:?}, number: {:?}",
            header.id(),
            header.number()
        );

        let mut merge_set = ghostdata
            .unordered_mergeset_without_selected_parent()
            .filter(|hash| self.storage.reachability_store.read().has(*hash).unwrap())
            .collect::<Vec<_>>()
            .into_iter();

        match inquirer::add_block(
            &mut stage,
            header.id(),
            ghostdata.selected_parent,
            &mut merge_set,
        ) {
            std::result::Result::Ok(_) => {}
            Err(e) => match e {
                ReachabilityError::DataInconsistency => {
                    warn!(
                        "the key {:?} was already processed, original error message: {:?}",
                        header.id(),
                        ReachabilityError::DataInconsistency
                    );
                }
                _ => {
                    panic!("failed to add block in batch for error: {:?}", e);
                }
            },
        }

        process_key_already_error(self.storage.relations_store.write().insert_batch(
            &mut batch,
            header.id(),
            BlockHashes::new(parents),
        ))
        .expect("failed to insert relations in batch");

        // Store header store
        process_key_already_error(self.storage.header_store.insert(
            header.id(),
            Arc::new(header.clone()),
            1,
        ))
        .expect("failed to insert header in batch");

        // the read lock will be updated to the write lock
        // and then write the batch
        // and then release the lock
        stage
            .commit(&mut batch)
            .expect("failed to write the stage reachability in batch");

        // write the data just one time
        self.storage
            .write_batch(batch)
            .expect("failed to write dag data in batch");

        drop(lock_guard);
        info!("finish writing the batch, head id: {:?}", header.id());

        Ok(())
    }

    pub fn commit(&mut self, header: BlockHeader) -> anyhow::Result<()> {
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

        if self.storage.reachability_store.read().get_reindex_root()? != header.pruning_point()
            && header.pruning_point() != HashValue::zero()
            && self
                .storage
                .reachability_store
                .read()
                .has(header.pruning_point())?
        {
            info!(
                "try to hint virtual selected parent, root index: {:?}",
                self.storage.reachability_store.read().get_reindex_root()
            );
            let hint_result = inquirer::hint_virtual_selected_parent(
                self.storage.reachability_store.write().deref_mut(),
                header.parent_hash(),
            );
            info!(
                "after hint virtual selected parent, root index: {:?}, hint result: {:?}",
                self.storage.reachability_store.read().get_reindex_root(),
                hint_result
            );
        }

        info!("start to commit via batch, header id: {:?}", header.id());

        // Create a DB batch writer
        let mut batch = WriteBatch::default();

        info!("start to commit via batch, header id: {:?}", header.id());
        let lock_guard = self.commit_lock.lock();

        // lock the dag data to write in batch, read lock.
        // the cache will be written at the same time
        // when the batch is written before flush to the disk and
        // if the writing process abort the starcoin process will/should restart.
        let mut stage = StagingReachabilityStore::new(
            self.storage.db.clone(),
            self.storage.reachability_store.upgradable_read(),
        );

        // Store ghostdata
        process_key_already_error(self.storage.ghost_dag_store.insert_batch(
            &mut batch,
            header.id(),
            ghostdata.clone(),
        ))
        .expect("failed to ghostdata in batch");

        // Update reachability store
        debug!(
            "start to update reachability data for block: {:?}, number: {:?}",
            header.id(),
            header.number()
        );

        let mut merge_set = ghostdata
            .unordered_mergeset_without_selected_parent()
            .filter(|hash| self.storage.reachability_store.read().has(*hash).unwrap())
            .collect::<Vec<_>>()
            .into_iter();

        match inquirer::add_block(
            &mut stage,
            header.id(),
            ghostdata.selected_parent,
            &mut merge_set,
        ) {
            std::result::Result::Ok(_) => {}
            Err(e) => match e {
                ReachabilityError::DataInconsistency => {
                    info!(
                        "the key {:?} was already processed, original error message: {:?}",
                        header.id(),
                        ReachabilityError::DataInconsistency
                    );
                }
                _ => {
                    panic!("failed to add block in batch for error: {:?}", e);
                }
            },
        }

        process_key_already_error(self.storage.relations_store.write().insert_batch(
            &mut batch,
            header.id(),
            BlockHashes::new(parents),
        ))
        .expect("failed to insert relations in batch");

        // Store header store
        process_key_already_error(self.storage.header_store.insert_batch(
            &mut batch,
            header.id(),
            Arc::new(header.clone()),
            1,
        ))
        .expect("failed to insert header in batch");

        // the read lock will be updated to the write lock
        // and then write the batch
        // and then release the lock
        stage
            .commit(&mut batch)
            .expect("failed to write the stage reachability in batch");

        // write the data just one time
        self.storage
            .write_batch(batch)
            .expect("failed to write dag data in batch");

        drop(lock_guard);
        info!("finish writing the batch, head id: {:?}", header.id());

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
        max_parents_count: u64,
    ) -> anyhow::Result<MineNewDagBlockInfo> {
        let mut dag_state = self.get_dag_state(previous_pruning_point)?;

        // filter
        if dag_state.tips.len() > max_parents_count as usize {
            dag_state.tips = dag_state
                .tips
                .into_iter()
                .sorted_by(|a, b| {
                    let a_blue_work = self
                        .storage
                        .ghost_dag_store
                        .get_blue_work(*a)
                        .unwrap_or_else(|e| {
                            panic!(
                                "the ghostdag data should be existed for {:?}, e: {:?}",
                                a, e
                            )
                        });
                    let b_blue_work = self
                        .storage
                        .ghost_dag_store
                        .get_blue_work(*b)
                        .unwrap_or_else(|e| {
                            panic!(
                                "the ghostdag data should be existed for {:?}, e: {:?}",
                                b, e
                            )
                        });
                    if a_blue_work == b_blue_work {
                        a.cmp(b)
                    } else {
                        b_blue_work.cmp(&a_blue_work)
                    }
                })
                .take(max_parents_count as usize)
                .collect();
        }

        let next_ghostdata = self.ghostdata(&dag_state.tips)?;

        let next_pruning_point = self.pruning_point_manager().next_pruning_point(
            previous_pruning_point,
            previous_ghostdata,
            &next_ghostdata,
            pruning_depth,
            pruning_finality,
        )?;

        if next_pruning_point == Hash::zero() || next_pruning_point == previous_pruning_point {
            info!(
                "tips: {:?}, the next pruning point is: {:?}, the current ghostdata's selected parent: {:?}, blue blocks are: {:?} and its red blocks are: {:?}",
                dag_state.tips, next_pruning_point, next_ghostdata.selected_parent, next_ghostdata.mergeset_blues, next_ghostdata.mergeset_reds.len(),
            );
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
            let pruned_ghostdata = self.ghost_dag_manager().ghostdag(&pruned_tips)?;
            let mergeset_blues = pruned_ghostdata.mergeset_blues.as_ref().clone();
            info!(
                "the pruning was triggered, previous tips: {:?}, the current tips: {:?}, the next pruning point is: {:?}, the current ghostdata's selected parent: {:?}, blue blocks are: {:?} and its red blocks are: {:?}",
                pruned_tips, dag_state.tips, next_pruning_point, pruned_ghostdata.selected_parent, pruned_ghostdata.mergeset_blues, pruned_ghostdata.mergeset_reds.len(),
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

    fn check_historical_block(
        &self,
        header: &BlockHeader,
        latest_pruning_point: Option<HashValue>,
    ) -> Result<bool, anyhow::Error> {
        if let Some(pruning_point) = latest_pruning_point {
            if pruning_point == HashValue::zero() {
                info!("pruning point is zero");
                Ok(true)
            } else if header.pruning_point() == pruning_point {
                info!(
                    "pruning point is the same as the latest pruning point, pruning point: {:?}",
                    pruning_point
                );
                Ok(false)
            } else if self.check_ancestor_of(header.pruning_point(), pruning_point)? {
                info!("pruning point is the ancestor of the latest pruning point, pruning point: {:?}, latest pruning point: {:?}", header.pruning_point(), pruning_point);
                Ok(true)
            } else {
                info!("pruning point is not the ancestor of the latest pruning point, pruning point: {:?}, latest pruning point: {:?}", header.pruning_point(), pruning_point);
                Ok(false)
            }
        } else {
            Ok(true)
        }
    }

    pub fn verify_and_ghostdata(
        &self,
        blue_blocks: &[BlockHeader],
        header: &BlockHeader,
        latest_pruning_point: Option<HashValue>,
    ) -> Result<GhostdagData, anyhow::Error> {
        if self.check_historical_block(header, latest_pruning_point)? {
            self.ghost_dag_manager()
                .verify_and_ghostdata(blue_blocks, header)
        } else {
            self.ghost_dag_manager().ghostdag(&header.parents())
        }
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
            .filter(
                |descendant| match self.check_ancestor_of(ancestor, *descendant) {
                    std::result::Result::Ok(result) => result,
                    Err(e) => {
                        warn!("Error checking ancestor relationship: {:?}", e);
                        false
                    }
                },
            )
            .collect::<Vec<_>>();
        anyhow::Ok(ReachabilityView {
            ancestor,
            descendants: de,
        })
    }
}
