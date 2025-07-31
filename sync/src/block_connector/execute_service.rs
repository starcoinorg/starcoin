use std::sync::Arc;

use anyhow::{Ok, Result};
use starcoin_chain::{verifier::FullVerifier, BlockChain};
use starcoin_chain::{ChainReader, ChainWriter};
use starcoin_config::{NodeConfig, TimeService};
use starcoin_dag::blockdag::BlockDAG;
use starcoin_logger::prelude::{debug, error, info};
use starcoin_service_registry::{
    bus::Bus, ActorService, EventHandler, ServiceContext, ServiceFactory,
};
use starcoin_storage::Storage;
use starcoin_types::system_events::{MinedBlock, NewDagBlock};

pub struct ExecuteService {
    time_service: Arc<dyn TimeService>,
    storage: Arc<Storage>,
    dag: BlockDAG,
}

impl ExecuteService {
    fn new(
        time_service: Arc<dyn TimeService>,
        storage: Arc<Storage>,
        dag: BlockDAG,
    ) -> ExecuteService {
        ExecuteService {
            time_service,
            storage,
            dag,
        }
    }
}

impl ServiceFactory<Self> for ExecuteService {
    fn create(ctx: &mut ServiceContext<Self>) -> Result<Self> {
        let config = ctx.get_shared::<Arc<NodeConfig>>()?;
        let time_service = config.net().time_service();
        let storage = ctx.get_shared::<Arc<Storage>>()?;
        let dag = ctx.get_shared::<BlockDAG>()?;

        Ok(Self::new(time_service, storage, dag))
    }
}

impl ActorService for ExecuteService {
    fn started(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.subscribe::<MinedBlock>();

        Ok(())
    }

    fn stopped(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.unsubscribe::<MinedBlock>();
        Ok(())
    }
}

impl EventHandler<Self, MinedBlock> for ExecuteService {
    fn handle_event(&mut self, msg: MinedBlock, ctx: &mut ServiceContext<Self>) {
        let MinedBlock(new_block) = msg;
        let id = new_block.header().id();
        debug!("try connect mined block: {}", id);

        let mut chain = BlockChain::new(
            self.time_service.clone(),
            new_block.header().parent_hash(),
            self.storage.clone(),
            None,
            self.dag.clone(),
        )
        .unwrap_or_else(|e| {
            panic!(
                "new block chain error when processing the mined block: {:?}",
                e
            )
        });

        let bus = ctx.bus_ref().clone();

        info!(
            "jacktest: verify, start to verify the mined block id: {:?}, number: {:?}",
            new_block.id(),
            new_block.header().number()
        );
        let verified_block =
            match chain.verify_with_verifier::<FullVerifier>(new_block.as_ref().clone()) {
                anyhow::Result::Ok(verified_block) => verified_block,
                Err(e) => {
                    error!(
                    "when verifying the mined block, failed to verify block error: {:?}, id: {:?}",
                    e,
                    new_block.id()
                );
                    return;
                }
            };
        info!(
            "jacktest: verify, end to verify the mined block id: {:?}, number: {:?}",
            new_block.id(),
            new_block.header().number()
        );

        info!(
            "jacktest: execute, start to execute the mined block id: {:?}, number: {:?}",
            new_block.id(),
            new_block.header().number()
        );
        let executed_block = match chain.execute(verified_block) {
            std::result::Result::Ok(executed_block) => executed_block,
            Err(e) => {
                error!(
                    "when executing the mined block, failed to execute block error: {:?}, id: {:?}",
                    e,
                    new_block.id()
                );
                return;
            }
        };
        info!(
            "jacktest: execute, end to execute the mined block id: {:?}, number: {:?}",
            new_block.id(),
            new_block.header().number()
        );

        info!(
            "jacktest: connect, start to connect the mined block id: {:?}, number: {:?}",
            new_block.id(),
            new_block.header().number()
        );
        match chain.connect(executed_block.clone()) {
            std::result::Result::Ok(_) => (),
            Err(e) => {
                error!("when connecting the mined block, failed to connect block error: {:?}, id: {:?}", e, new_block.id());
                return;
            }
        }
        info!(
            "jacktest: connect, end to execute the mined block id: {:?}, number: {:?}",
            new_block.id(),
            new_block.header().number()
        );

        info!(
            "jacktest: new dag block, start to broadcast new dag block id: {:?}, number: {:?}",
            new_block.id(),
            new_block.header().number()
        );
        let _ = bus.broadcast(NewDagBlock {
            executed_block: Arc::new(executed_block.clone()),
        });
    }
}
