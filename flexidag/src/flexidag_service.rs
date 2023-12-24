use std::sync::Arc;

use anyhow::{Ok, Result};
use starcoin_config::NodeConfig;
use starcoin_crypto::HashValue;
use starcoin_dag::blockdag::BlockDAG;
use starcoin_service_registry::{
    ActorService, ServiceContext, ServiceFactory,
};
use starcoin_storage::{flexi_dag::DagTips, Storage, SyncFlexiDagStore};

#[derive(Debug, Clone)]
pub struct MergesetBlues {
    pub selected_parent: HashValue,
    pub mergeset_blues: Vec<HashValue>,
}

pub struct FlexidagService {
    dag: BlockDAG,
    // dag_accumulator: Option<MerkleAccumulator>,
    tip_info: Option<DagTips>,
    storage: Arc<Storage>,
}

impl ServiceFactory<Self> for FlexidagService {
    fn create(ctx: &mut ServiceContext<FlexidagService>) -> Result<Self> {
        let storage = ctx.get_shared::<Arc<Storage>>()?;
        let config = ctx.get_shared::<Arc<NodeConfig>>()?;
        let dag = BlockDAG::try_init_with_storage(storage.clone())?;
        ctx.put_shared(dag.clone())?;
        let tip_info = storage.get_dag_tips()?;
        Ok(Self {
            dag,
            tip_info,
            storage,
        })
    }
}

impl ActorService for FlexidagService {
    fn started(&mut self, _ctx: &mut ServiceContext<Self>) -> Result<()> {
        // ctx.subscribe::<NewHeadBlock>();
        Ok(())
    }

    fn stopped(&mut self, _ctx: &mut ServiceContext<Self>) -> Result<()> {
        // ctx.unsubscribe::<NewHeadBlock>();
        Ok(())
    }
}
