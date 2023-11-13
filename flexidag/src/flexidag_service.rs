use std::{
    collections::{BTreeSet, BinaryHeap},
    sync::Arc,
};

use anyhow::{anyhow, bail, Error, Ok, Result};
use starcoin_accumulator::{accumulator_info::AccumulatorInfo, Accumulator, MerkleAccumulator, node::AccumulatorStoreType};
use starcoin_config::{NodeConfig, TimeService};
use starcoin_consensus::{dag::types::ghostdata::GhostdagData, BlockDAG};
use starcoin_crypto::HashValue;
use starcoin_service_registry::{
    ActorService, ServiceContext, ServiceFactory, ServiceHandler, ServiceRequest,
};
use starcoin_storage::{
    flexi_dag::{SyncFlexiDagSnapshot, SyncFlexiDagSnapshotHasher},
    storage::CodecKVStore,
    BlockStore, Storage, SyncFlexiDagStore, block_info::BlockInfoStore, Store,
};
use starcoin_types::{block::BlockHeader, header::DagHeader, startup_info, dag_block::KTotalDifficulty};

#[derive(Debug, Clone)]
pub struct DumpTipsToAccumulator {
    pub block_header: BlockHeader,
    pub current_head_block_id: HashValue,
    pub k_total_difficulty: KTotalDifficulty,
}

impl ServiceRequest for DumpTipsToAccumulator {
    type Response = anyhow::Result<()>;
}

#[derive(Debug, Clone)]
pub struct UpdateDagTips {
    pub block_header: BlockHeader,
    pub current_head_block_id: HashValue,
    pub k_total_difficulty: KTotalDifficulty,
}

impl ServiceRequest for UpdateDagTips {
    type Response = anyhow::Result<()>;
}

#[derive(Debug, Clone)]
pub struct GetDagTips;

impl ServiceRequest for GetDagTips {
    type Response = anyhow::Result<Option<Vec<HashValue>>>;
}

#[derive(Debug, Clone)]
pub struct GetDagAccumulatorInfo;

impl ServiceRequest for GetDagAccumulatorInfo {
    type Response = anyhow::Result<Option<AccumulatorInfo>>;
}

#[derive(Debug, Clone)]
pub struct GetDagAccumulatorLeafDetail {
    pub leaf_index: u64,
    pub batch_size: u64,
}

#[derive(Debug, Clone)]
pub struct DagAccumulatorLeafDetail {
    pub accumulator_root: HashValue,
    pub tips: Vec<HashValue>,
}

impl ServiceRequest for GetDagAccumulatorLeafDetail {
    type Response = anyhow::Result<Vec<DagAccumulatorLeafDetail>>;
}

#[derive(Debug, Clone)]
pub struct GetDagBlockParents {
    pub block_id: HashValue,
}

#[derive(Debug, Clone)]
pub struct DagBlockParents {
    pub parents: Vec<HashValue>,
}

impl ServiceRequest for GetDagBlockParents {
    type Response = anyhow::Result<DagBlockParents>;
}

#[derive(Debug, Clone)]
pub struct GetDagAccumulatorLeaves {
    pub leaf_index: u64,
    pub batch_size: u64,
    pub reverse: bool,
}

#[derive(Debug, Clone)]
pub struct DagAccumulatorLeaf {
    pub leaf_index: u64,
    pub dag_accumulator_root: HashValue,
}

impl ServiceRequest for GetDagAccumulatorLeaves {
    type Response = anyhow::Result<Vec<DagAccumulatorLeaf>>;
}

#[derive(Debug, Clone)]
pub struct AddToDag {
    pub block_header: BlockHeader,
}

#[derive(Debug, Clone)]
pub struct MergesetBlues {
    pub selected_parent: HashValue,
    pub mergeset_blues: Vec<HashValue>,
}

impl ServiceRequest for AddToDag {
    type Response = anyhow::Result<MergesetBlues>;
}

#[derive(Debug, Clone)]
pub struct ForkDagAccumulator {
    pub new_blocks: Vec<HashValue>,
    pub dag_accumulator_index: u64,
    pub block_header_id: HashValue,
}

impl ServiceRequest for ForkDagAccumulator {
    type Response = anyhow::Result<AccumulatorInfo>;
}

#[derive(Debug, Clone)]
pub struct FinishSync {
    pub dag_accumulator_info: AccumulatorInfo,
}

impl ServiceRequest for FinishSync {
    type Response = anyhow::Result<()>;
}

pub struct TipInfo {
    tips: Option<Vec<HashValue>>, // some is for dag or the state of the chain is still in old version
    k_total_difficulties: BTreeSet<KTotalDifficulty>,
}

pub struct FlexidagService {
    dag: Option<BlockDAG>,
    dag_accumulator: Option<MerkleAccumulator>,
    tip_info: Option<TipInfo>,
    storage: Arc<Storage>,
}

impl FlexidagService {
    pub fn add_to_dag(&mut self, header: BlockHeader) -> Result<Arc<GhostdagData>> {
        let dag = match &mut self.dag {
            Some(dag) => dag,
            None => bail!("dag is none"),
        };
        match dag.get_ghostdag_data(header.id()) {
            std::result::Result::Ok(ghost_dag_data) => Ok(ghost_dag_data),
            Err(_) => std::result::Result::Ok(Arc::new(
                // jacktest: TODO:add_to_dag  should not use parents hash since the block header has them
                dag.add_to_dag(DagHeader::new(header.clone()))?,
            )),
        }
    }

    fn create_snapshot_by_tips(tips: Vec<HashValue>, head_block_id: HashValue, storage: Arc<Storage>) -> Result<(HashValue, SyncFlexiDagSnapshotHasher)> {
        let mut k_total_difficulties = BTreeSet::new();
        tips.iter().try_for_each(|block_id| {
            k_total_difficulties.insert(KTotalDifficulty {
                head_block_id: block_id.clone(),
                total_difficulty: storage.get_block_info(block_id.clone()).expect("block info should not be none").ok_or_else(|| anyhow!("block info should not be none"))?.total_difficulty,
            });
            Ok(())
        })?;

        let snapshot_hasher = SyncFlexiDagSnapshotHasher {
            child_hashes: tips,
            head_block_id,
            k_total_difficulties,
        };

        Ok((BlockDAG::calculate_dag_accumulator_key(&snapshot_hasher)?, snapshot_hasher))
    }

    fn merge_from_big_dag(&mut self, msg: ForkDagAccumulator) -> Result<AccumulatorInfo> {
        let dag_accumulator = self.dag_accumulator.as_mut().ok_or_else(|| anyhow!("the dag accumulator should not be none"))?;
        if dag_accumulator.num_leaves() != msg.dag_accumulator_index {
            bail!("cannot merge dag accumulator since its number is not the same as other");
        }
        let tip_info = self.tip_info.as_mut().ok_or_else(|| anyhow!("the tips should not be none"))?;
        msg.new_blocks.iter().for_each(|block_id| {
            if !tip_info.tips.as_ref().expect("tips should not be none").contains(block_id) {
                tip_info.tips.as_mut().expect("tips should not be none").push(block_id.clone());
            }
        });
  
        let (key, snaphot_hasher) = Self::create_snapshot_by_tips(tip_info.tips.as_ref().expect("tips should not be none").clone(), msg.block_header_id, self.storage.clone())?;
        dag_accumulator.append(&vec![key])?;
        let dag_accumulator_info = dag_accumulator.get_info();
        self.storage.get_accumulator_snapshot_storage().put(key, snaphot_hasher.to_snapshot(dag_accumulator_info.clone()))?;
        dag_accumulator.flush()?;
        Ok(dag_accumulator_info)
    }

    fn merge_from_small_dag(&mut self, msg: ForkDagAccumulator) -> Result<AccumulatorInfo> {
        let dag_accumulator = self
            .dag_accumulator
            .as_mut()
            .ok_or_else(|| anyhow!("dag accumulator is none"))?;
        // fetch the block in the dag according to the dag accumulator index
        let previous_key = dag_accumulator.get_leaf(msg.dag_accumulator_index - 1)?
            .ok_or_else(|| anyhow!("the dag snapshot hash is none"))?;

        let current_key = dag_accumulator.get_leaf(msg.dag_accumulator_index)?
            .ok_or_else(|| anyhow!("the dag snapshot hash is none"))?;

        let pre_snapshot = self
            .storage
            .get_accumulator_snapshot_storage()
            .get(previous_key)?
            .ok_or_else(|| anyhow!("the dag snapshot is none"))?;

        let current_snapshot = self
            .storage
            .get_accumulator_snapshot_storage()
            .get(current_key)?
            .ok_or_else(|| anyhow!("the dag snapshot is none"))?;

        // fork the dag accumulator according to the ForkDagAccumulator.dag_accumulator_index
        let fork = dag_accumulator.fork(Some(pre_snapshot.accumulator_info));

        let mut new_blocks = msg.new_blocks;
        current_snapshot.child_hashes.iter().for_each(|block_id| {
            if !new_blocks.contains(block_id) {
                new_blocks.push(block_id.clone());
            }
        });

        let (key, snaphot_hasher) = Self::create_snapshot_by_tips(new_blocks, msg.block_header_id, self.storage.clone())?;
        fork.append(&vec![key])?;
        let dag_accumulator_info = fork.get_info();
        self.storage.get_accumulator_snapshot_storage().put(key, snaphot_hasher.to_snapshot(dag_accumulator_info.clone()))?;
        fork.flush()?;
        Ok(dag_accumulator_info)  
    }

}

impl ServiceFactory<Self> for FlexidagService {
    fn create(ctx: &mut ServiceContext<FlexidagService>) -> Result<Self> {
        let storage = ctx.get_shared::<Arc<Storage>>()?;
        let config = ctx.get_shared::<Arc<NodeConfig>>()?;
        let (dag, dag_accumulator) =
            BlockDAG::try_init_with_storage(storage.clone(), config.clone())?;
        let tip_info = dag_accumulator.as_ref().map(|accumulator| {
            let tips_index = accumulator.num_leaves();
            let tips_key = accumulator
                .get_leaf(tips_index)
                .expect("failed to read the dag snapshot hash")
                .expect("the dag snapshot hash is none");
            let snapshot = storage
                .get_accumulator_snapshot_storage()
                .get(tips_key)
                .expect("failed to read the snapsho object")
                .expect("dag snapshot object is none");
            TipInfo {
                tips: Some(snapshot.child_hashes),
                k_total_difficulties: snapshot.k_total_difficulties,
            }
        });
        Ok(Self {
            dag,
            dag_accumulator,
            tip_info,
            storage: storage.clone(),
        })
    }
}

impl ActorService for FlexidagService {
    fn started(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        // ctx.subscribe::<NewHeadBlock>();
        Ok(())
    }

    fn stopped(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        // ctx.unsubscribe::<NewHeadBlock>();
        Ok(())
    }
}

// send this message after minting a new block
// and the block was committed
// and startup info was updated
impl ServiceHandler<Self, DumpTipsToAccumulator> for FlexidagService {
    fn handle(
        &mut self,
        msg: DumpTipsToAccumulator,
        ctx: &mut ServiceContext<FlexidagService>,
    ) -> Result<()> {
        let storage = ctx.get_shared::<Arc<Storage>>()?;
        if self.tip_info.is_none() {
            let config = ctx.get_shared::<Arc<NodeConfig>>()?;
            let (dag, dag_accumulator) = BlockDAG::try_init_with_storage(storage.clone(), config)?;
            if dag.is_none() {
                Ok(()) // the chain is still in single chain
            } else {
                // initialize the dag data, the chain will be the dag chain at next block
                self.dag = dag;
                self.dag_accumulator = dag_accumulator;
                self.tip_info = Some(TipInfo {
                    tips: Some(vec![msg.block_header.id()]),
                    k_total_difficulties: [msg.k_total_difficulty].into_iter().collect(),
                });
                self.storage = storage.clone();
                Ok(())
            }
        } else {
            // the chain had became the flexidag chain
            let mut tip_info = self
                .tip_info
                .take()
                .expect("the tips should not be none in this branch");
            let snapshot_hasher = SyncFlexiDagSnapshotHasher {
                child_hashes: tip_info.tips.expect("the tips should not be none"),
                head_block_id: msg.current_head_block_id,
                k_total_difficulties: tip_info.k_total_difficulties,
            };
            let key = BlockDAG::calculate_dag_accumulator_key(&snapshot_hasher)?;
            let dag = self
                .dag_accumulator
                .as_mut()
                .expect("the tips is not none but the dag accumulator is none");
            dag.append(&vec![key])?;
            storage.get_accumulator_snapshot_storage().put(
                key,
                snapshot_hasher.to_snapshot(dag.get_info()) 
            )?;
            dag.flush()?;
            self.tip_info = Some(TipInfo {
                tips: Some(vec![msg.block_header.id()]),
                k_total_difficulties: [msg.k_total_difficulty].into_iter().collect(),
            });
            self.storage = storage.clone();
            Ok(())
        }
    }
}

impl ServiceHandler<Self, UpdateDagTips> for FlexidagService {
    fn handle(
        &mut self,
        msg: UpdateDagTips,
        ctx: &mut ServiceContext<FlexidagService>,
    ) -> Result<()> {
        let header = msg.block_header;
        match &mut self.tip_info {
            Some(tip_info) => {
                if !tip_info.tips.as_ref().expect("tips should not be none").contains(&header.id()) {
                    tip_info.tips.as_mut().expect("tips should not be none").push(header.id());
                    tip_info.k_total_difficulties.insert(KTotalDifficulty {
                        head_block_id: msg.k_total_difficulty.head_block_id,
                        total_difficulty: msg.k_total_difficulty.total_difficulty,
                    });
                }
                Ok(())
            }
            None => {
                let storage = ctx.get_shared::<Arc<Storage>>()?;
                let config = ctx.get_shared::<Arc<NodeConfig>>()?;
                if header.number() == storage.dag_fork_height(config.net().id().clone()) {
                    let (dag, dag_accumulator) =
                        BlockDAG::try_init_with_storage(storage.clone(), config)?;
                    if dag.is_none() {
                        Ok(()) // the chain is still in single chain
                    } else {
                        // initialize the dag data, the chain will be the dag chain at next block
                        self.dag = dag;
                        self.tip_info = Some(TipInfo {
                            tips: Some(vec![header.id()]),
                            k_total_difficulties: [msg.k_total_difficulty]
                                .into_iter()
                                .collect(),
                        });
                        self.dag_accumulator = dag_accumulator;

                        storage
                            .get_startup_info()?
                            .map(|mut startup_info| {
                                startup_info.dag_main = Some(header.id());
                                storage.save_startup_info(startup_info)
                            })
                            .expect("starup info should not be none")
                    }
                } else {
                    Ok(()) // drop the block, the chain is still in single chain
                }
            }
        }
    }
}

impl ServiceHandler<Self, GetDagTips> for FlexidagService {
    fn handle(
        &mut self,
        _msg: GetDagTips,
        _ctx: &mut ServiceContext<FlexidagService>,
    ) -> Result<Option<Vec<HashValue>>> {
        Ok(self.tip_info.as_ref().ok_or_else(|| anyhow!("tip info is none"))?.tips.clone())
    }
}

impl ServiceHandler<Self, GetDagAccumulatorInfo> for FlexidagService {
    fn handle(
        &mut self,
        _msg: GetDagAccumulatorInfo,
        _ctx: &mut ServiceContext<FlexidagService>,
    ) -> Result<Option<AccumulatorInfo>> {
        Ok(self
            .dag_accumulator
            .as_ref()
            .map(|dag_accumulator_info| dag_accumulator_info.get_info()))
    }
}

impl ServiceHandler<Self, GetDagAccumulatorLeaves> for FlexidagService {
    fn handle(
        &mut self,
        msg: GetDagAccumulatorLeaves,
        _ctx: &mut ServiceContext<FlexidagService>,
    ) -> Result<Vec<DagAccumulatorLeaf>> {
        match &self.dag_accumulator {
            Some(dag_accumulator) => {
                let end_index = std::cmp::min(
                    msg.leaf_index + msg.batch_size - 1,
                    dag_accumulator.num_leaves() - 1,
                );
                let mut result = vec![];
                for index in msg.leaf_index..=end_index {
                    let real_index = if msg.reverse {
                        end_index - index + 1
                    } else {
                        index
                    };
                    let key = dag_accumulator
                        .get_leaf(real_index)?
                        .ok_or_else(|| anyhow!("the dag snapshot hash is none"))?;
                    let snaptshot = self
                        .storage
                        .get_accumulator_snapshot_storage()
                        .get(key)?
                        .expect("the snapshot should not be none");
                    result.push(DagAccumulatorLeaf {
                        leaf_index: real_index,
                        dag_accumulator_root: snaptshot.accumulator_info.accumulator_root,
                    });
                }
                Ok(result)
            }
            None => bail!("dag accumulator is none"),
        }
    }
}

impl ServiceHandler<Self, GetDagBlockParents> for FlexidagService {
    fn handle(
        &mut self,
        msg: GetDagBlockParents,
        _ctx: &mut ServiceContext<FlexidagService>,
    ) -> Result<DagBlockParents> {
        match &self.dag {
            Some(dag) => Ok(DagBlockParents {
                parents: dag.get_parents(msg.block_id)?,
            }),
            None => bail!("dag is none"),
        }
    }
}

impl ServiceHandler<Self, GetDagAccumulatorLeafDetail> for FlexidagService {
    fn handle(
        &mut self,
        msg: GetDagAccumulatorLeafDetail,
        _ctx: &mut ServiceContext<FlexidagService>,
    ) -> Result<Vec<DagAccumulatorLeafDetail>> {
        match &self.dag_accumulator {
            Some(dag_accumulator) => {
                let end_index = std::cmp::min(
                    msg.leaf_index + msg.batch_size - 1,
                    dag_accumulator.num_leaves() - 1,
                );
                let mut details = vec![];
                let snapshot_storage = self.storage.get_accumulator_snapshot_storage();
                for index in msg.leaf_index..=end_index {
                    let key = dag_accumulator
                        .get_leaf(index)?
                        .ok_or_else(|| anyhow!("the dag snapshot hash is none"))?;
                    let snapshot = snapshot_storage
                        .get(key)?
                        .ok_or_else(|| anyhow!("the dag snapshot is none"))?;
                    details.push(DagAccumulatorLeafDetail {
                        accumulator_root: snapshot.accumulator_info.accumulator_root,
                        tips: snapshot.child_hashes,
                    });
                }
                Ok(details)
            }
            None => bail!("dag accumulator is none"),
        }
    }
}

impl ServiceHandler<Self, AddToDag> for FlexidagService {
    fn handle(
        &mut self,
        msg: AddToDag,
        _ctx: &mut ServiceContext<FlexidagService>,
    ) -> Result<MergesetBlues> {
        let ghost_dag_data = self.add_to_dag(msg.block_header)?;
        Ok(MergesetBlues {
            selected_parent: ghost_dag_data.selected_parent,
            mergeset_blues: ghost_dag_data.mergeset_blues.as_ref().clone(),
        })
    }
}

impl ServiceHandler<Self, ForkDagAccumulator> for FlexidagService {
    fn handle(
        &mut self,
        msg: ForkDagAccumulator,
        _ctx: &mut ServiceContext<FlexidagService>,
    ) -> Result<AccumulatorInfo> {
        let dag_accumulator = self
            .dag_accumulator
            .as_ref()
            .ok_or_else(|| anyhow!("dag accumulator is none"))?;

        if msg.dag_accumulator_index > dag_accumulator.num_leaves() {
            self.merge_from_big_dag(msg)
        } else {
            self.merge_from_small_dag(msg)
        }
    }
}

impl ServiceHandler<Self, FinishSync> for FlexidagService {
    fn handle(
        &mut self,
        msg: FinishSync,
        _ctx: &mut ServiceContext<FlexidagService>,
    ) -> Result<()> {
        let dag_accumulator = self.dag_accumulator.as_mut().ok_or_else(|| anyhow!("the dag_accumulator is none when sync finish"))?;
        let local_info = dag_accumulator.get_info();
        if msg.dag_accumulator_info.get_num_leaves() < local_info.get_num_leaves() {
            let start_idnex = msg.dag_accumulator_info.get_num_leaves(); 
            let new_dag_accumulator = MerkleAccumulator::new_with_info(msg.dag_accumulator_info, self.storage.get_accumulator_store(AccumulatorStoreType::SyncDag));
            for index in start_idnex..local_info.get_num_leaves() {
                let key = dag_accumulator.get_leaf(index)?.ok_or_else(|| anyhow!("the dag_accumulator leaf is none when sync finish"))?;
                new_dag_accumulator.append(&[key])?;
            }
            self.dag_accumulator = Some(new_dag_accumulator);
            Ok(())
        } else {
            self.dag_accumulator = Some(MerkleAccumulator::new_with_info(msg.dag_accumulator_info, self.storage.get_accumulator_store(AccumulatorStoreType::SyncDag)));
            Ok(())
        }
    }
}