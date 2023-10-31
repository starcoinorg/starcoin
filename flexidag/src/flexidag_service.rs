use std::sync::Arc;

use anyhow::{anyhow, Result, Ok, bail, Error};
use starcoin_accumulator::{Accumulator, MerkleAccumulator, accumulator_info::AccumulatorInfo};
use starcoin_config::NodeConfig;
use starcoin_consensus::BlockDAG;
use starcoin_crypto::HashValue;
use starcoin_service_registry::{ActorService, ServiceContext, ServiceFactory, ServiceHandler, ServiceRequest};
use starcoin_storage::{storage::CodecKVStore, flexi_dag::SyncFlexiDagSnapshot, Storage, SyncFlexiDagStore, BlockStore};
use starcoin_types::{block::BlockHeader, startup_info};

#[derive(Debug, Clone)]
pub struct DumpTipsToAccumulator {
    block_header: BlockHeader,
}

impl ServiceRequest for DumpTipsToAccumulator {
    type Response = anyhow::Result<()>;
}

#[derive(Debug, Clone)]
pub struct UpdateDagTips {
    pub block_header: BlockHeader,
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

pub struct FlexidagService {
    dag: Option<BlockDAG>,
    dag_accumulator: Option<MerkleAccumulator>,
    tips: Option<Vec<HashValue>>, // some is for dag or the state of the chain is still in old version
    storage: Arc<Storage>,
}

impl ServiceFactory<Self> for FlexidagService {
    fn create(ctx: &mut ServiceContext<FlexidagService>) -> Result<Self> {
        let storage = ctx.get_shared::<Arc<Storage>>()?;
        let config = ctx.get_shared::<Arc<NodeConfig>>()?;
        let (dag, dag_accumulator) = BlockDAG::try_init_with_storage(storage.clone(), config.clone())?;
        let tips = dag_accumulator.as_ref().map(|accumulator| {
            let tips_index = accumulator.num_leaves();
            let tips_key = accumulator
                .get_leaf(tips_index)
                .expect("failed to read the dag snapshot hash")
                .expect("the dag snapshot hash is none");
            storage
                .get_accumulator_snapshot_storage()
                .get(tips_key)
                .expect("failed to read the snapsho object")
                .expect("dag snapshot object is none")
                .child_hashes
        });
        Ok(Self {
            dag,
            dag_accumulator,
            tips,
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
    fn handle(&mut self, msg: DumpTipsToAccumulator, ctx: &mut ServiceContext<FlexidagService>) -> Result<()> {
        let storage = ctx.get_shared::<Arc<Storage>>()?;
        if self.tips.is_none() {
            let config = ctx.get_shared::<Arc<NodeConfig>>()?;
            let (dag, dag_accumulator) = BlockDAG::try_init_with_storage(storage, config)?;
            if dag.is_none() {
                Ok(()) // the chain is still in single chain
            } else {
                // initialize the dag data, the chain will be the dag chain at next block
                self.dag = dag;
                self.tips = Some(vec![msg.block_header.id()]);
                self.dag_accumulator = dag_accumulator;

                Ok(())
            }
        } else {
            // the chain became the flexidag chain
            let tips = self.tips.take().expect("the tips should not be none in this branch");
            let key = BlockDAG::calculate_dag_accumulator_key(tips.clone())?;
            let dag = self.dag_accumulator.as_mut().expect("the tips is not none but the dag accumulator is none");
            dag.append(&vec![key])?;
            storage.get_accumulator_snapshot_storage().put(key, SyncFlexiDagSnapshot {
                child_hashes: tips,
                accumulator_info: dag.get_info(),
            })?;
            dag.flush()?;
            self.tips = Some(vec![]);
            Ok(())
        }
    }
}

impl ServiceHandler<Self, UpdateDagTips> for FlexidagService {
    fn handle(&mut self, msg: UpdateDagTips, ctx: &mut ServiceContext<FlexidagService>) -> Result<()> {
        let header = msg.block_header;
        match self.tips.clone() {
            Some(mut tips) => {
                if !tips.contains(&header.id()) {
                    tips.push(header.id());
                    self.tips = Some(tips);
                }
                Ok(())
            }
            None => {
                let storage = ctx.get_shared::<Arc<Storage>>()?;
                let config = ctx.get_shared::<Arc<NodeConfig>>()?;
                if header.number() == storage.dag_fork_height(config.net().id().clone()) {
                    let (dag, dag_accumulator) = BlockDAG::try_init_with_storage(storage.clone(), config)?;
                    if dag.is_none() {
                        Ok(()) // the chain is still in single chain
                    } else {
                        // initialize the dag data, the chain will be the dag chain at next block
                        self.dag = dag;
                        self.tips = Some(vec![header.id()]);
                        self.dag_accumulator = dag_accumulator;

                        storage.get_startup_info()?.map(|mut startup_info| {
                            startup_info.dag_main = Some(header.id());
                            storage.save_startup_info(startup_info)
                        }).expect("starup info should not be none")
                    }
                } else {
                    Ok(()) // drop the block, the chain is still in single chain
                }
            }
        }
    }
}

impl ServiceHandler<Self, GetDagTips> for FlexidagService {
    fn handle(&mut self, _msg: GetDagTips, _ctx: &mut ServiceContext<FlexidagService>) -> Result<Option<Vec<HashValue>>> {
        Ok(self.tips.clone())
    }
}

impl ServiceHandler<Self, GetDagAccumulatorInfo> for FlexidagService {
    fn handle(&mut self, _msg: GetDagAccumulatorInfo, _ctx: &mut ServiceContext<FlexidagService>) -> Result<Option<AccumulatorInfo>> {
        Ok(self.dag_accumulator.as_ref().map(|dag_accumulator_info| {
            dag_accumulator_info.get_info()
        }))
    }
}

impl ServiceHandler<Self, GetDagAccumulatorLeaves> for FlexidagService {
    fn handle(&mut self, msg: GetDagAccumulatorLeaves, _ctx: &mut ServiceContext<FlexidagService>) -> Result<Vec<DagAccumulatorLeaf>> {
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
                    let key = dag_accumulator.get_leaf(real_index)?.ok_or_else(|| anyhow!("the dag snapshot hash is none"))?;
                    let snaptshot = self.storage.get_accumulator_snapshot_storage().get(key)?.expect("the snapshot should not be none");
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
    fn handle(&mut self, msg: GetDagBlockParents, _ctx: &mut ServiceContext<FlexidagService>) -> Result<DagBlockParents> {
        match &self.dag {
            Some(dag) => Ok(DagBlockParents { parents: dag.get_parents(msg.block_id)? } ) ,
            None => bail!("dag is none"),
        }
    }
}

impl ServiceHandler<Self, GetDagAccumulatorLeafDetail> for FlexidagService {
    fn handle(&mut self, msg: GetDagAccumulatorLeafDetail, _ctx: &mut ServiceContext<FlexidagService>) -> Result<Vec<DagAccumulatorLeafDetail>> {
        match &self.dag_accumulator {
            Some(dag_accumulator) => {
                let end_index = std::cmp::min(
                    msg.leaf_index + msg.batch_size - 1,
                    dag_accumulator.num_leaves() - 1,
                );
                let mut details = vec![];
                let snapshot_storage = self.storage.get_accumulator_snapshot_storage(); 
                for index in msg.leaf_index..=end_index {
                    let key = dag_accumulator.get_leaf(index)?.ok_or_else(|| anyhow!("the dag snapshot hash is none"))?;
                    let snapshot = snapshot_storage.get(key)?.ok_or_else(|| anyhow!("the dag snapshot is none"))?;
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