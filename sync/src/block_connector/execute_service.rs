use std::sync::Arc;

use anyhow::{Ok, Result};
use starcoin_chain::{verifier::FullVerifier, BlockChain};
use starcoin_chain::{ChainReader, ChainWriter};
use starcoin_chain_api::ExecutedBlock;
use starcoin_config::{NodeConfig, TimeService};
use starcoin_dag::blockdag::BlockDAG;
use starcoin_logger::prelude::{debug, error, info};
use starcoin_service_registry::{
    bus::Bus, ActorService, EventHandler, ServiceContext, ServiceFactory,
};
use starcoin_storage::block_info::BlockInfoStore;
use starcoin_storage::Storage;
use starcoin_sync_api::{PeerNewBlock, SelectHeaderState};
use starcoin_types::block::Block;
use starcoin_types::consensus_header::ConsensusHeader;
use starcoin_types::system_events::{MinedBlock, NewDagBlock, NewDagBlockFromPeer};

use crate::sync::CheckSyncEvent;

pub struct ExecuteService {
    time_service: Arc<dyn TimeService>,
    storage: Arc<Storage>,
    dag: BlockDAG,
}

impl ExecuteService {
    fn new(time_service: Arc<dyn TimeService>, storage: Arc<Storage>, dag: BlockDAG) -> Self {
        Self {
            time_service,
            storage,
            dag,
        }
    }

    fn execute(&self, new_block: Block) -> Result<ExecutedBlock> {
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

        info!(
            "jacktest: verify, start to verify the mined block id: {:?}, number: {:?}",
            new_block.id(),
            new_block.header().number()
        );
        let id = new_block.id();
        let verified_block = match chain.verify_with_verifier::<FullVerifier>(new_block) {
            anyhow::Result::Ok(verified_block) => verified_block,
            Err(e) => {
                error!(
                    "when verifying the mined block, failed to verify block error: {:?}, id: {:?}",
                    e, id,
                );
                return Err(e);
            }
        };
        info!(
            "jacktest: verify, end to verify the mined block id: {:?}, number: {:?}",
            verified_block.block.id(),
            verified_block.block.header().number()
        );

        let executed_block = match chain.execute(verified_block) {
            std::result::Result::Ok(executed_block) => executed_block,
            Err(e) => {
                error!(
                    "when executing the mined block, failed to execute block error: {:?}, id: {:?}",
                    e, id,
                );
                return Err(e);
            }
        };
        info!(
            "jacktest: execute, end to execute the mined block id: {:?}, number: {:?}",
            executed_block.block.id(),
            executed_block.block.header().number()
        );

        match chain.connect(executed_block.clone()) {
            std::result::Result::Ok(_) => (),
            Err(e) => {
                error!("when connecting the mined block, failed to connect block error: {:?}, id: {:?}", e, executed_block.block.id());
                return Err(e);
            }
        }
        info!(
            "jacktest: connect, end to execute the mined block id: {:?}, number: {:?}",
            executed_block.block.id(),
            executed_block.block().header().number()
        );

        Ok(executed_block)
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
        ctx.subscribe::<PeerNewBlock>();
        Ok(())
    }

    fn stopped(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.unsubscribe::<MinedBlock>();
        ctx.unsubscribe::<PeerNewBlock>();
        Ok(())
    }
}

impl EventHandler<Self, PeerNewBlock> for ExecuteService {
    fn handle_event(&mut self, msg: PeerNewBlock, ctx: &mut ServiceContext<Self>) {
        let block = msg.get_block();

        let block_info = self
            .storage
            .get_block_info(block.header().parent_hash())
            .expect("get block info error in execute service");
        if block_info.is_none() {
            ctx.broadcast(CheckSyncEvent::default());
            return;
        }

        for parent in block.header().parents() {
            let block_info = self
                .storage
                .get_block_info(parent)
                .expect("get block info for parents error in execute service");
            if block_info.is_none() {
                ctx.broadcast(CheckSyncEvent::default());
                return;
            }
        }

        match self.execute(msg.get_block().clone()) {
            std::result::Result::Ok(executed_block) => {
                ctx.broadcast(NewDagBlockFromPeer {
                    executed_block: Arc::new(executed_block.header().clone()),
                });
                let bus = ctx.bus_ref().clone();
                let _ = bus.broadcast(SelectHeaderState::new(
                    msg.get_peer_id(),
                    msg.get_block().clone(),
                ));
            }
            Err(e) => error!(
                "execute a peer block {:?} error: {}",
                msg.get_block().id(),
                e
            ),
        }
    }
}

impl EventHandler<Self, MinedBlock> for ExecuteService {
    fn handle_event(&mut self, msg: MinedBlock, ctx: &mut ServiceContext<Self>) {
        let MinedBlock(new_block) = msg;
        let id = new_block.header().id();
        debug!("try connect mined block: {}", id);

        match self.execute(new_block.as_ref().clone()) {
            std::result::Result::Ok(executed_block) => {
                let bus = ctx.bus_ref().clone();
                let _ = bus.broadcast(NewDagBlock {
                    executed_block: Arc::new(executed_block.clone()),
                });
            }
            Err(e) => error!("execute a mined block {:?} error: {}", new_block.id(), e),
        }
    }
}
