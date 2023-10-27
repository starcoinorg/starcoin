use std::sync::Arc;

use anyhow::Result;
use starcoin_accumulator::{MerkleAccumulator, Accumulator, node::AccumulatorStoreType};
use starcoin_config::NodeConfig;
use starcoin_consensus::BlockDAG;
use starcoin_crypto::HashValue;
use starcoin_service_registry::{ServiceFactory, ActorService, ServiceContext};
use starcoin_storage::{Store, storage_macros, storage::CodecKVStore};
use starcoin_types::startup_info;


pub struct FlexidagService {
    dag: Option<BlockDAG>,
    dag_accumulator: Option<MerkleAccumulator>,
    tips: Option<Vec<HashValue>>, // some is for dag or the state of the chain is still in old version 
}

impl ServiceFactory<Self> for FlexidagService {
    fn create(ctx: &mut ServiceContext<FlexidagService>) -> Result<Self> {
        let storage = ctx.get_shared::<Arc<dyn Store>>()?;
        let config = ctx.get_shared::<Arc<NodeConfig>>()?;
        let dag = BlockDAG::init_with_storage(storage.clone(), config.clone())?; 
        let startup_info = storage.get_startup_info()?.expect("StartupInfo should exist.");
        let dag_accumulator = startup_info.get_dag_main().map(|key| {
            storage.get_dag_accumulator_info(key).expect("the key of dag accumulator is not none but it is none in storage")
        }).map(|dag_accumulator_info| {
            MerkleAccumulator::new_with_info(dag_accumulator_info, storage.get_accumulator_store(AccumulatorStoreType::SyncDag))
        });
        let tips = dag_accumulator.as_ref().map(|accumulator| {
            let tips_index = accumulator.num_leaves();
            let tips_key = accumulator
            .get_leaf(tips_index)
            .expect("failed to read the dag snapshot hash")
            .expect("the dag snapshot hash is none");
            storage.get_accumulator_snapshot_storage()
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