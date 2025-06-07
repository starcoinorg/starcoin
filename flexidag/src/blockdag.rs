use super::reachability::{inquirer, reachability_service::MTReachabilityService};
use super::types::ghostdata::GhostdagData;
use crate::block_depth::block_depth_info::BlockDepthManagerT;
use crate::consensusdb::consensus_block_depth::DbBlockDepthInfoStore;
use crate::consensusdb::consensus_pruning_info::{
    PruningPointInfo, PruningPointInfoReader, PruningPointInfoWriter,
};
use crate::consensusdb::consensus_state::{
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
use crate::ghostdag::protocol::{GhostdagManager, KStore};
use crate::prune::pruning_point_manager::PruningPointManagerT;
use crate::reachability::reachability_service::ReachabilityService;
use crate::reachability::ReachabilityError;
use crate::{process_key_already_error, GetAbsentBlock, GetAbsentBlockResult};
use anyhow::{bail, ensure, format_err, Ok};
use itertools::Itertools;
use parking_lot::{Mutex, RwLockUpgradableReadGuard};
use rocksdb::WriteBatch;
use starcoin_config::miner_config::{G_MAX_PARENTS_COUNT, G_MERGE_DEPTH};
use starcoin_config::temp_dir;
use starcoin_crypto::{HashValue as Hash, HashValue};
use starcoin_logger::prelude::{debug, error, info, warn};
use starcoin_state_api::AccountStateReader;
use starcoin_statedb::ChainStateDB;
use starcoin_storage::{IntoSuper, Storage};
use starcoin_types::account_config::genesis_address;
use starcoin_types::block::BlockHeader;
use starcoin_types::{
    blockhash::{BlockHashes, KType},
    consensus_header::ConsensusHeader,
};
use starcoin_vm_runtime::force_upgrade_management::get_force_upgrade_block_number;
use starcoin_vm_types::on_chain_resource::Epoch;
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
pub type BlockDepthManager =
    BlockDepthManagerT<DbBlockDepthInfoStore, DbReachabilityStore, DbGhostdagStore>;

pub struct MineNewDagBlockInfo {
    pub tips: Vec<HashValue>,
    pub ghostdata: GhostdagData,
    pub pruning_point: HashValue,
}

#[derive(Clone)]
pub struct BlockDAG {
    pub storage: FlexiDagStorage,
    ghostdag_manager: DbGhostdagManager,
    pruning_point_manager: PruningPointManager,
    block_depth_manager: BlockDepthManager,
    commit_lock: Arc<Mutex<FlexiDagStorage>>,
}

impl BlockDAG {
    pub fn create_blockdag(dag_storage: FlexiDagStorage) -> Self {
        Self::new(DEFAULT_GHOSTDAG_K, G_MERGE_DEPTH, dag_storage)
    }

    pub fn new(k: KType, merge_depth: u64, db: FlexiDagStorage) -> Self {
        let ghostdag_store = db.ghost_dag_store.clone();
        let header_store = db.header_store.clone();
        let relations_store = db.relations_store.clone();
        let reachability_store = db.reachability_store.clone();
        let reachability_service = MTReachabilityService::new(reachability_store);
        let k_store = Arc::new(KStore::new(k));
        let ghostdag_manager = DbGhostdagManager::new(
            k_store,
            ghostdag_store.clone(),
            relations_store,
            header_store,
            reachability_service.clone(),
        );
        let pruning_point_manager =
            PruningPointManager::new(reachability_service.clone(), ghostdag_store.clone());
        let block_depth_manager = BlockDepthManager::new(
            db.block_depth_info_store.clone(),
            reachability_service,
            ghostdag_store,
            merge_depth,
        );
        Self {
            ghostdag_manager,
            storage: db.clone(),
            pruning_point_manager,
            block_depth_manager,
            commit_lock: Arc::new(Mutex::new(db)),
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
        Ok(Self::new(k, G_MERGE_DEPTH, dag_storage))
    }

    pub fn create_for_testing_with_k_and_merge_depth(
        k: KType,
        merge_depth: u64,
    ) -> anyhow::Result<Self> {
        let dag_storage =
            FlexiDagStorage::create_from_path(temp_dir(), FlexiDagStorageConfig::default())?;
        Ok(Self::new(k, merge_depth, dag_storage))
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

    pub fn check_ancestor_of_chain(
        &self,
        ancestor: Hash,
        descendant: Hash,
    ) -> anyhow::Result<bool> {
        inquirer::is_chain_ancestor_of(
            &*self.storage.reachability_store.read(),
            ancestor,
            descendant,
        )
        .map_err(|e| e.into())
    }

    pub fn check_ancestor_of(&self, ancestor: Hash, descendant: Hash) -> anyhow::Result<bool> {
        if ancestor == Hash::zero() {
            return Ok(true);
        }
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
    pub fn ghostdata_by_hashes(
        &self,
        hashes: &[HashValue],
    ) -> anyhow::Result<Vec<Option<Arc<GhostdagData>>>> {
        let mut results = Vec::with_capacity(hashes.len());
        for &hash in hashes {
            match self.storage.ghost_dag_store.get_data(hash) {
                Result::Ok(data) => results.push(Some(data)),
                Err(StoreError::KeyNotFound(_)) => results.push(None),
                Err(e) => return Err(e.into()),
            }
        }
        Ok(results)
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

        let mut merge_set = ghostdata.unordered_mergeset_without_selected_parent();

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

        let mut relations_write = self.storage.relations_store.write();
        process_key_already_error(relations_write.insert_batch(
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
        let stage_write = stage
            .commit(&mut batch)
            .expect("failed to write the stage reachability in batch");

        // write the data just one time
        self.storage
            .write_batch(batch)
            .expect("failed to write dag data in batch");

        info!("finish writing the batch, head id: {:?}", header.id());

        drop(stage_write);
        drop(relations_write);
        drop(lock_guard);

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

        let mut merge_set = ghostdata.unordered_mergeset_without_selected_parent();

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

        let mut relations_write = self.storage.relations_store.write();
        process_key_already_error(relations_write.insert_batch(
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
        let stage_write = stage
            .commit(&mut batch)
            .expect("failed to write the stage reachability in batch");

        // write the data just one time
        self.storage
            .write_batch(batch)
            .expect("failed to write dag data in batch");

        info!("finish writing the batch, head id: {:?}", header.id());

        drop(stage_write);
        drop(relations_write);
        drop(lock_guard);

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

                let mut tips_in_order = merged_tips
                    .into_iter()
                    .map(|hash: Hash| {
                        let data = self.storage.ghost_dag_store.get_compact_data(hash)?;
                        std::result::Result::Ok((hash, data))
                    })
                    .collect::<std::result::Result<Vec<_>, StoreError>>()?;
                tips_in_order.sort_by(|a, b| match b.1.blue_work.cmp(&a.1.blue_work) {
                    std::cmp::Ordering::Equal => match b.1.blue_score.cmp(&a.1.blue_score) {
                        std::cmp::Ordering::Equal => b.0.cmp(&a.0),
                        other => other,
                    },
                    other => other,
                });

                writer.insert(
                    hash,
                    DagState {
                        tips: tips_in_order
                            .into_iter()
                            .map(|(id, _)| id)
                            .take(usize::try_from(G_MAX_PARENTS_COUNT)?)
                            .collect(),
                    },
                )?;
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
        max_parents_count: u64,
        genesis_id: HashValue,
    ) -> anyhow::Result<MineNewDagBlockInfo> {
        let mut dag_state = self.get_dag_state(previous_pruning_point)?;

        let latest_pruning_point = if let Some(pruning_point_info) = self
            .storage
            .pruning_point_store
            .read()
            .get_pruning_point_info()?
        {
            pruning_point_info.pruning_point
        } else {
            warn!(
                "failed to get the pruning point info by hash: {:?}, the block should be re-executed",
                previous_pruning_point
            );
            previous_pruning_point
        };

        let pruned_tips = self.pruning_point_manager().prune(
            &dag_state,
            previous_pruning_point,
            latest_pruning_point,
        )?;
        let mut next_pruning_point = latest_pruning_point;
        dag_state.tips = pruned_tips;

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
        match self.storage.ghost_dag_store.insert(
            self.ghost_dag_manager()
                .find_selected_parent(dag_state.tips.iter().cloned())?,
            Arc::new(next_ghostdata.clone()),
        ) {
            std::result::Result::Ok(_) => (),
            Err(e) => match e {
                StoreError::KeyAlreadyExists(_) => (),
                _ => return Err(e.into()),
            },
        }

        if next_pruning_point == Hash::zero() {
            next_pruning_point = genesis_id;
        }

        anyhow::Ok(MineNewDagBlockInfo {
            tips: dag_state.tips,
            ghostdata: next_ghostdata,
            pruning_point: next_pruning_point,
        })
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

    pub fn validate_pruning_point(
        &self,
        selected_header: Hash,
        pruning_point: Hash,
        pruning_depth: u64,
    ) -> anyhow::Result<()> {
        // if !self
        //     .reachability_service()
        //     .is_chain_ancestor_of(pruning_point, selected_header)
        // {
        //     warn!("the pruning point is not the ancestor of the selected header");
        //     return Err(anyhow::anyhow!(
        //         "the pruning point: {:?} is not the ancestor of the selected header: {:?}",
        //         pruning_point,
        //         selected_header
        //     ));
        // }

        let selected_header_blue_score = self
            .storage
            .ghost_dag_store
            .get_blue_score(selected_header)?;
        let pruning_point_blue_score =
            self.storage.ghost_dag_store.get_blue_score(pruning_point)?;

        if selected_header_blue_score < pruning_point_blue_score + pruning_depth {
            warn!("the pruning point blue score is not correct");
            return Err(anyhow::anyhow!("the pruning point blue score: {:?} + pruning depth: {:?} is larger than selected header blue score: {:?}", pruning_point_blue_score, pruning_depth, selected_header_blue_score));
        }

        anyhow::Ok(())
    }

    pub fn block_depth_manager(&self) -> BlockDepthManager {
        self.block_depth_manager.clone()
    }

    // pub fn generate_the_block_depth(
    //     &self,
    //     merge_depth_root: Hash,
    //     finality_point: Hash,
    //     ghostdata: &GhostdagData,
    // ) -> anyhow::Result<BlockDepthInfo> {
    //     // let merge_depth_root = self
    //     //     .block_depth_manager
    //     //     .calc_merge_depth_root(ghostdata, pruning_point)?;
    //     // if merge_depth_root == Hash::zero() {
    //     //     return anyhow::Ok(BlockDepthInfo {
    //     //         merge_depth_root,
    //     //         finality_point: Hash::zero(),
    //     //     });
    //     // }
    //     // let finality_point = self.block_depth_manager.calc_finality_point(
    //     //     ghostdata,
    //     //     pruning_point,
    //     //     finality_depth,
    //     // )?;
    //     self.storage.block_depth_info_store.insert(
    //         ghostdata.selected_parent,
    //         BlockDepthInfo {
    //             merge_depth_root,
    //             finality_point,
    //         },
    //     )?;
    //     info!(
    //         "the merge depth root is: {:?}, the finality point is: {:?}",
    //         merge_depth_root, finality_point
    //     );
    //     Ok(BlockDepthInfo {
    //         merge_depth_root,
    //         finality_point,
    //     })
    // }

    pub fn check_bounded_merge_depth(
        &self,
        ghostdata: &GhostdagData,
        merge_depth_root: Hash,
    ) -> anyhow::Result<()> {
        // let merge_depth_root = self
        //     .block_depth_manager
        //     .get_block_depth_info(ghostdata.selected_parent)?
        //     .ok_or_else(|| format_err!("failed to get block depth info"))?
        //     .merge_depth_root;

        let mut kosherizing_blues: Option<Vec<Hash>> = None;

        for red in ghostdata.mergeset_reds.iter().copied() {
            if self
                .reachability_service()
                .is_dag_ancestor_of(merge_depth_root, red)
            {
                continue;
            }
            // Lazy load the kosherizing blocks since this case is extremely rare
            if kosherizing_blues.is_none() {
                kosherizing_blues = Some(
                    self.block_depth_manager
                        .kosherizing_blues(ghostdata, merge_depth_root)
                        .collect(),
                );
            }
            if !self.reachability_service().is_dag_ancestor_of_any(
                red,
                &mut kosherizing_blues.as_ref().unwrap().iter().copied(),
            ) {
                warn!("failed to verify the bounded merge depth, the header refers too many bad red blocks, red: {:?}, kosherizing blues: {:?}", red, kosherizing_blues);
            }
        }

        Ok(())
    }

    pub fn remove_bounded_merge_breaking_parents(
        &self,
        mut parents: Vec<Hash>,
        mut ghostdata: GhostdagData,
        pruning_point: Hash,
        merge_depth_root: Hash,
    ) -> anyhow::Result<(Vec<Hash>, GhostdagData)> {
        if pruning_point == Hash::zero() {
            return anyhow::Ok((parents, ghostdata));
        }
        // let merge_depth_root = self
        //     .block_depth_manager
        //     .calc_merge_depth_root(&ghostdata, pruning_point)
        //     .map_err(|e| anyhow::anyhow!("Failed to calculate merge depth root: {}", e))?;
        if merge_depth_root == Hash::zero() {
            return anyhow::Ok((parents, ghostdata));
        }
        // debug!("merge depth root: {:?}", merge_depth_root);
        let mut kosherizing_blues: Option<Vec<Hash>> = None;
        let mut bad_reds = Vec::new();

        // Find red blocks violating the merge bound and which are not kosherized by any blue
        for red in ghostdata.mergeset_reds.iter().copied() {
            if self
                .reachability_service()
                .is_dag_ancestor_of(merge_depth_root, red)
            {
                continue;
            }
            // Lazy load the kosherizing blocks since this case is extremely rare
            if kosherizing_blues.is_none() {
                kosherizing_blues = Some(
                    self.block_depth_manager
                        .kosherizing_blues(&ghostdata, merge_depth_root)
                        .collect(),
                );
            }
            if !self.reachability_service().is_dag_ancestor_of_any(
                red,
                &mut kosherizing_blues.as_ref().unwrap().iter().copied(),
            ) {
                bad_reds.push(red);
            }
        }

        if !bad_reds.is_empty() {
            // Remove all parents which lead to merging a bad red
            parents.retain(|&h| {
                !self
                    .reachability_service()
                    .is_any_dag_ancestor(&mut bad_reds.iter().copied(), h)
            });
            // Recompute ghostdag data since parents changed
            ghostdata = self.ghostdag_manager.ghostdag(&parents)?;
        }

        anyhow::Ok((parents, ghostdata))
    }
    pub fn reachability_store(
        &self,
    ) -> Arc<parking_lot::lock_api::RwLock<parking_lot::RawRwLock, DbReachabilityStore>> {
        self.storage.reachability_store.clone()
    }

    pub fn reachability_service(&self) -> MTReachabilityService<DbReachabilityStore> {
        self.pruning_point_manager().reachability_service()
    }

    // return true the block processing will be going into the single chain logic,
    // which means that this is a historical block that converge to the next pruning point
    // return false it will be going into the ghost logic,
    // which means that this is a new block that will be added by the ghost protocol that enhance the parallelism of the block processing.
    // for vega, the special situation is:
    // the pruning logic was delivered after vega running for a long time, so the historical block will be processed by the single chain logic.
    fn check_historical_block(
        &self,
        header: &BlockHeader,
        latest_pruning_point: Option<HashValue>,
    ) -> Result<bool, anyhow::Error> {
        if let Some(pruning_point) = latest_pruning_point {
            match (
                header.pruning_point() == HashValue::zero(),
                pruning_point == HashValue::zero(),
            ) {
                (true, true) => {
                    if header.chain_id().is_vega() {
                        Ok(true)
                    } else {
                        Ok(false)
                    }
                }
                (true, false) => Ok(true),
                (false, true) => Ok(false),
                (false, false) => {
                    if header.pruning_point() == pruning_point {
                        Ok(false)
                    } else if self.check_ancestor_of(header.pruning_point(), pruning_point)? {
                        Ok(true)
                    } else {
                        Ok(false)
                    }
                }
            }
        } else {
            Ok(false)
        }
    }

    pub fn verify_and_ghostdata(
        &self,
        blue_blocks: &[BlockHeader],
        header: &BlockHeader,
        latest_pruning_point: Option<HashValue>,
    ) -> Result<GhostdagData, anyhow::Error> {
        info!(
            "checking historical block: header pruning point: {:?}, latest pruning point: {:?}",
            header.pruning_point(),
            latest_pruning_point
        );

        // 3500000 is the block number that the pruning logic was delivered in vega
        // when the logic is delivered onto the vega,  we should check a number larger than 3500000 to ignore this logic
        if header.chain_id().is_vega()
            && header.number() < 3500000
            && self.check_historical_block(header, latest_pruning_point)?
        {
            info!(
                "the block is a historical block, the header id: {:?}",
                header.id()
            );
            self.ghost_dag_manager()
                .build_ghostdata(blue_blocks, header)
        } else {
            info!(
                "the block is not a historical block, the header id: {:?}",
                header.id()
            );
            self.ghost_dag_manager().ghostdag(&header.parents())
        }
    }

    pub fn check_upgrade(
        &self,
        main: &BlockHeader,
        genesis_id: HashValue,
        storage: Arc<Storage>,
    ) -> anyhow::Result<()> {
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

        // update k
        if main.number() >= get_force_upgrade_block_number(&main.chain_id()) {
            let state_root = main.state_root();
            let state_db = ChainStateDB::new(storage.clone().into_super_arc(), Some(state_root));
            let account_reader = AccountStateReader::new(&state_db);
            let epoch = account_reader
                .get_resource::<Epoch>(genesis_address())?
                .ok_or_else(|| format_err!("Epoch is none."))?;

            self.ghost_dag_manager()
                .update_k(u16::try_from(epoch.max_uncles_per_block())?);
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

    pub fn get_absent_blocks(&self, req: GetAbsentBlock) -> anyhow::Result<GetAbsentBlockResult> {
        let relation = self.storage.relations_store.read();
        let mut result = HashSet::from_iter(req.absent_id.clone());
        let mut roud = req.absent_id.into_iter().collect::<HashSet<HashValue>>();
        for _ in 0..req.exp {
            let next_round = roud
                .into_iter()
                .map(|parent| relation.get_parents(parent))
                .collect::<std::result::Result<Vec<_>, _>>()?
                .into_iter()
                .flat_map(|v| (*v).clone())
                .collect::<HashSet<HashValue>>();
            roud = next_round.difference(&result).cloned().collect();
            result = result.union(&next_round).cloned().collect();
        }

        drop(relation);

        Ok(GetAbsentBlockResult {
            absent_blocks: result.into_iter().collect(),
        })
    }

    pub async fn generate_pruning_point(
        &self,
        header: &BlockHeader,
        pruning_depth: u64,
        pruning_finality: u64,
        genesis_id: Hash,
    ) -> anyhow::Result<()> {
        let previous_ghostdata = if header.pruning_point() == HashValue::zero() {
            self.ghostdata_by_hash(genesis_id)?.ok_or_else(|| format_err!("Cannot find ghostdata by hash when trying to calculate the pruning point: {:?}", header.id()))?
        } else {
            self.ghostdata_by_hash(header.pruning_point())?.ok_or_else(|| format_err!("Cannot find ghostdata by hash when trying to calculate the pruning point: {:?}", header.id()))?
        };
        let next_ghostdata = self.ghostdata_by_hash(header.id())?.ok_or_else(|| {
            format_err!(
                "Cannot find ghostdata by hash when trying to calculate the pruning point: {:?}",
                header.id()
            )
        })?;

        let reader = self.storage.pruning_point_store.upgradable_read();

        let current_pruning_point_info = match reader.get_pruning_point_info() {
            std::result::Result::Ok(info) => match info {
                Some(info) => info,
                None => {
                    let writer = RwLockUpgradableReadGuard::upgrade(reader);

                    writer.insert(PruningPointInfo {
                        pruning_point: if header.pruning_point() == HashValue::zero() {
                            genesis_id
                        } else {
                            header.pruning_point()
                        },
                    })?;

                    drop(writer);
                    return Ok(());
                }
            },
            Err(e) => match e {
                StoreError::KeyNotFound(_) => {
                    let writer = RwLockUpgradableReadGuard::upgrade(reader);
                    writer.insert(PruningPointInfo {
                        pruning_point: if header.pruning_point() == HashValue::zero() {
                            genesis_id
                        } else {
                            header.pruning_point()
                        },
                    })?;

                    drop(writer);
                    return Ok(());
                }
                _ => {
                    error!("Failed to get pruning point info: {:?}", e);
                    return Err(e.into());
                }
            },
        };

        let current_pruning_point_ghostdata = self
            .storage
            .ghost_dag_store
            .get_data(current_pruning_point_info.pruning_point)?;

        let new_pruning_point = self.pruning_point_manager().next_pruning_point(
            current_pruning_point_info.pruning_point,
            previous_ghostdata.as_ref(),
            next_ghostdata.as_ref(),
            pruning_depth,
            pruning_finality,
        )?;

        let new_pruning_point_ghostdata =
            self.storage
                .ghost_dag_store
                .get_data(if new_pruning_point == HashValue::zero() {
                    genesis_id
                } else {
                    new_pruning_point
                })?;

        let should_update = match current_pruning_point_ghostdata
            .blue_score
            .cmp(&new_pruning_point_ghostdata.blue_score)
        {
            std::cmp::Ordering::Less => true,
            std::cmp::Ordering::Equal => {
                match new_pruning_point.cmp(&current_pruning_point_info.pruning_point) {
                    std::cmp::Ordering::Less => false,
                    std::cmp::Ordering::Equal => false,
                    std::cmp::Ordering::Greater => true,
                }
            }
            std::cmp::Ordering::Greater => false,
        };

        if !should_update {
            drop(reader);
            return Ok(());
        }

        let writer = RwLockUpgradableReadGuard::upgrade(reader);

        writer.insert(PruningPointInfo {
            pruning_point: new_pruning_point,
        })?;

        drop(writer);

        Ok(())
    }
}
