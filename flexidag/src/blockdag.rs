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
use crate::prune::pruning_point_manager::PruningPointManagerT;
use crate::{process_key_already_error, reachability};
use anyhow::{bail, Ok};
use starcoin_config::temp_dir;
use starcoin_crypto::{HashValue as Hash, HashValue};
use starcoin_logger::prelude::{debug, info};
use starcoin_types::block::BlockHeader;
use starcoin_types::{
    blockhash::{BlockHashes, KType},
    consensus_header::ConsensusHeader,
};
use std::ops::DerefMut;
use std::sync::Arc;

pub const DEFAULT_GHOSTDAG_K: KType = 8u16;
pub const DEFAULT_PRUNING_DEPTH: u64 = 185798;
pub const DEFAULT_FINALITY_DEPTH: u64 = 86400;

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
        Ok(Self::new(DEFAULT_GHOSTDAG_K, dag_storage))
    }

    pub fn create_for_testing_with_parameters(k: KType) -> anyhow::Result<Self> {
        let dag_storage =
            FlexiDagStorage::create_from_path(temp_dir(), FlexiDagStorageConfig::default())?;
        Ok(BlockDAG::new(k, dag_storage))
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
        println!("jacktest: 1");
        let genesis_id = genesis.id();
        let origin = genesis.parent_hash();

        inquirer::init(self.storage.reachability_store.write().deref_mut(), origin)?;
        println!("jacktest: 2");

        self.storage
            .relations_store
            .write()
            .insert(origin, BlockHashes::new(vec![]))?;
        println!("jacktest: 3");

        self.commit(genesis, origin)?;
        println!("jacktest: 4");
        self.save_dag_state(DagState {
            tips: vec![genesis_id],
            pruning_point: genesis_id,
        })?;
        println!("jacktest: 5");
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

    pub fn commit(&mut self, header: BlockHeader, origin: HashValue) -> anyhow::Result<()> {
        info!(
            "start to commit header: {:?}, number: {:?}",
            header.id(),
            header.number()
        );
        println!(
            "jacktest: start to commit header: {:?}, number: {:?}, origin: {:?}",
            header,
            header.number(),
            origin,
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
                    println!("jacktest: commit1");
                    Arc::new(self.ghostdag_manager.genesis_ghostdag_data(&header))
                } else {
                    println!("jacktest: commit2");
                    let ghostdata = self.ghostdag_manager.ghostdag(&parents)?;
                    println!("jacktest: commit3");
                    Arc::new(ghostdata)
                }
            }
            Some(ghostdata) => ghostdata,
        };
        println!("jacktest: commit4");
        // Store ghostdata
        process_key_already_error(
            self.storage
                .ghost_dag_store
                .insert(header.id(), ghostdata.clone()),
        )?;
        println!("jacktest: commit5");

        // Update reachability store
        debug!(
            "start to update reachability data for block: {:?}, number: {:?}",
            header.id(),
            header.number()
        );
        let reachability_store = self.storage.reachability_store.clone();
        println!("jacktest: commit6");

        let mut merge_set = ghostdata
            .unordered_mergeset_without_selected_parent()
            .filter(|hash| self.storage.reachability_store.read().has(*hash).unwrap())
            .collect::<Vec<_>>()
            .into_iter();
        println!("jacktest: commit7");
        let add_block_result = {
            let mut reachability_writer = reachability_store.write();
            inquirer::add_block(
                reachability_writer.deref_mut(),
                header.id(),
                ghostdata.selected_parent,
                &mut merge_set,
            )
        };
        println!("jacktest: commit8");
        match add_block_result {
            Result::Ok(_) => (),
            Err(reachability::ReachabilityError::DataInconsistency) => {
                println!("jacktest: commit9");
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
                println!("jacktest: commit10");
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

        println!("jacktest: commit11");
        process_key_already_error(
            self.storage
                .relations_store
                .write()
                .insert(header.id(), BlockHashes::new(parents)),
        )?;
        println!("jacktest: commit12");
        // Store header store
        process_key_already_error(self.storage.header_store.insert(
            header.id(),
            Arc::new(header),
            1,
        ))?;
        println!("jacktest: commit13");
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

    pub fn get_dag_state(&self) -> anyhow::Result<DagState> {
        Ok(self.storage.state_store.read().get_state()?)
    }

    pub fn save_dag_state(&self, state: DagState) -> anyhow::Result<()> {
        self.storage.state_store.write().insert(state)?;
        Ok(())
    }

    pub fn ghost_dag_manager(&self) -> DbGhostdagManager {
        self.ghostdag_manager.clone()
    }

    pub fn calc_mergeset_and_tips(
        &self,
        _pruning_depth: u64,
        _pruning_finality: u64,
    ) -> anyhow::Result<MineNewDagBlockInfo> {
        let dag_state = self.get_dag_state()?;
        let ghostdata = self.ghost_dag_manager().ghostdag(&dag_state.tips)?;
        println!(
            "jacktest: dag state: {:?}, ghost data: {:?}",
            dag_state, ghostdata
        );
        anyhow::Ok(MineNewDagBlockInfo {
            tips: dag_state.tips,
            blue_blocks: (*ghostdata.mergeset_blues).clone(),
            pruning_point: HashValue::zero(),
        })

        // let next_pruning_point = self.pruning_point_manager().next_pruning_point(
        //     &dag_state,
        //     &ghostdata,
        //     pruning_depth,
        //     pruning_finality,
        // )?;
        // if next_pruning_point == dag_state.pruning_point {
        //     anyhow::Ok(MineNewDagBlockInfo {
        //         tips: dag_state.tips,
        //         blue_blocks: (*ghostdata.mergeset_blues).clone(),
        //         pruning_point: next_pruning_point,
        //     })
        // } else {
        //     let pruned_tips = self
        //         .pruning_point_manager()
        //         .prune(&dag_state, next_pruning_point)?;
        //     let mergeset_blues = (*self
        //         .ghost_dag_manager()
        //         .ghostdag(&pruned_tips)?
        //         .mergeset_blues)
        //         .clone();
        //     anyhow::Ok(MineNewDagBlockInfo {
        //         tips: pruned_tips,
        //         blue_blocks: mergeset_blues,
        //         pruning_point: next_pruning_point,
        //     })
        // }
    }

    fn verify_pruning_point(
        &self,
        pruning_depth: u64,
        pruning_finality: u64,
        block_header: &BlockHeader,
        genesis_id: HashValue,
    ) -> anyhow::Result<()> {
        let dag_state = DagState {
            tips: block_header.parents(),
            pruning_point: block_header.pruning_point(),
        };
        let ghostdata = self.ghost_dag_manager().ghostdag(&block_header.parents())?;
        let next_pruning_point = self.pruning_point_manager().next_pruning_point(
            &dag_state,
            &ghostdata,
            pruning_depth,
            pruning_finality,
        )?;

        if (block_header.chain_id().is_vega()
            || block_header.chain_id().is_proxima()
            || block_header.chain_id().is_halley())
            && block_header.pruning_point() == HashValue::zero()
        {
            if next_pruning_point == genesis_id {
                return anyhow::Ok(());
            } else {
                bail!(
                    "pruning point is not correct, it should update the next pruning point: {}",
                    next_pruning_point
                );
            }
        }
        if next_pruning_point != block_header.pruning_point() {
            bail!("pruning point is not correct, the local next pruning point is {}, but the block header pruning point is {}", next_pruning_point, block_header.pruning_point());
        }
        anyhow::Ok(())
    }

    pub fn verify(
        &self,
        pruning_depth: u64,
        pruning_finality: u64,
        block_header: &BlockHeader,
        genesis_id: HashValue,
    ) -> anyhow::Result<()> {
        self.verify_pruning_point(pruning_depth, pruning_finality, block_header, genesis_id)
    }
}
