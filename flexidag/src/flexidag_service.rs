use std::sync::Arc;

use anyhow::Result;
use starcoin_accumulator::{node::AccumulatorStoreType, Accumulator, MerkleAccumulator, accumulator_info::AccumulatorInfo};
use starcoin_config::NodeConfig;
use starcoin_consensus::BlockDAG;
use starcoin_crypto::HashValue;
use starcoin_service_registry::{ActorService, ServiceContext, ServiceFactory, ServiceHandler, ServiceRequest};
use starcoin_storage::{storage::{CodecKVStore, self}, Store, flexi_dag::SyncFlexiDagSnapshot, Storage, SyncFlexiDagStore};
use starcoin_types::block::BlockHeader;

#[derive(Debug, Clone)]
pub struct UpdateDagTips;

impl ServiceRequest for UpdateDagTips {
    type Response = anyhow::Result<()>;
}

#[derive(Debug, Clone)]
pub struct NewDagBlock {
    block_header: BlockHeader,
}

impl ServiceRequest for NewDagBlock {
    type Response = anyhow::Result<()>;
}

pub struct FlexidagService {
    dag: Option<BlockDAG>,
    dag_accumulator: Option<MerkleAccumulator>,
    tips: Option<Vec<HashValue>>, // some is for dag or the state of the chain is still in old version
}

impl ServiceFactory<Self> for FlexidagService {
    fn create(ctx: &mut ServiceContext<FlexidagService>) -> Result<Self> {
        let storage = ctx.get_shared::<Arc<Storage>>()?;
        let config = ctx.get_shared::<Arc<NodeConfig>>()?;
        let dag = BlockDAG::init_with_storage(storage.clone(), config.clone())?;
        let startup_info = storage
            .get_startup_info()?
            .expect("StartupInfo should exist.");
        let dag_accumulator = startup_info
            .get_dag_main()
            .map(|key| {
                storage
                    .get_dag_accumulator_info(key)
                    .expect("the key of dag accumulator is not none but it is none in storage")
            })
            .map(|dag_accumulator_info| {
                MerkleAccumulator::new_with_info(
                    dag_accumulator_info,
                    storage.get_accumulator_store(AccumulatorStoreType::SyncDag),
                )
            });
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

impl ServiceHandler<Self, UpdateDagTips> for FlexidagService {
    fn handle(&mut self, _msg: UpdateDagTips, ctx: &mut ServiceContext<FlexidagService>) -> Result<()> {
        if self.tips.is_none() {
            Ok(())
        } else {
            let storage = ctx.get_shared::<Arc<Storage>>()?;
            let tips = self.tips.take().expect("the tips should not be none in this branch");
            self.tips = Some(vec![]);
            let key = BlockDAG::calculate_dag_accumulator_key(tips.clone())?;
            let dag = self.dag_accumulator.as_mut().expect("dag accumulator is none").fork(None);
            dag.append(&vec![key])?;
            storage.get_accumulator_snapshot_storage().put(key, SyncFlexiDagSnapshot {
                child_hashes: tips.clone(),
                accumulator_info: dag.get_info(),
            })?;
            dag.flush()?;
            self.dag_accumulator = Some(dag);
            Ok(())
        }
    }
}

// To flush the dag accumulator and tips, NewDagBlock must be sent after startup info is updated 
impl ServiceHandler<Self, NewDagBlock> for FlexidagService {
    fn handle(&mut self, msg: NewDagBlock, ctx: &mut ServiceContext<FlexidagService>) -> Result<()> {
        if self.tips.is_none() {
            let storage = ctx.get_shared::<Arc<Storage>>()?;
            let config = ctx.get_shared::<Arc<NodeConfig>>()?;
            let dag = BlockDAG::init_with_storage(storage, config)?;
            if dag.is_none() {
                Ok(())
            } else {
                self.dag = dag;
                self.tips = Some(vec![msg.block_header.id()]);
                self.dag_accumulator = Some(MerkleAccumulator::new_with_info(AccumulatorInfo::new(accumulator_root, frozen_subtree_roots, num_leaves, num_nodes), node_store));
            }
        } else {
            Ok(())
        }
    }
}