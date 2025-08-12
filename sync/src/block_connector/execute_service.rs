use std::sync::Arc;

use anyhow::{Ok, Result};
use network_api::PeerId;
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

enum ExecuteBlockFrom {
    LocalMinedBlock,
    PeerMinedBlock(PeerId),
}

struct ExecutedBlockInfo {
    executed_block: ExecutedBlock,
    from: ExecuteBlockFrom,
}

pub struct ExecuteService {
    time_service: Arc<dyn TimeService>,
    storage: Arc<Storage>,
    dag: BlockDAG,
    receiver: crossbeam::channel::Receiver<Arc<ExecutedBlockInfo>>,
    sender: crossbeam::channel::Sender<Arc<ExecutedBlockInfo>>,
}

impl ExecuteService {
    fn new(time_service: Arc<dyn TimeService>, storage: Arc<Storage>, dag: BlockDAG) -> Self {
        let (sender, receiver) = crossbeam::channel::unbounded::<Arc<ExecutedBlockInfo>>();
        Self {
            time_service,
            storage,
            dag,
            receiver,
            sender,
        }
    }

    fn execute(
        new_block: Block,
        time_service: Arc<dyn TimeService>,
        storage: Arc<Storage>,
        dag: BlockDAG,
    ) -> Result<ExecutedBlock> {
        info!(
            "[BlockProcess] now start to execute the block: {:?}",
            new_block.id()
        );
        let mut chain = BlockChain::new(
            time_service,
            new_block.header().parent_hash(),
            storage,
            None,
            dag,
        )
        .unwrap_or_else(|e| {
            panic!(
                "new block chain error when processing the mined block: {:?}",
                e
            )
        });

        let id = new_block.id();
        info!("[BlockProcess] now verify the block: {:?}", new_block.id());
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
            "[BlockProcess] now execute the block: {:?}",
            verified_block.block.id()
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
            "[BlockProcess] now connect the block: {:?}",
            executed_block.block.id()
        );

        match chain.connect(executed_block.clone()) {
            std::result::Result::Ok(_) => (),
            Err(e) => {
                error!("when connecting the mined block, failed to connect block error: {:?}, id: {:?}", e, executed_block.block.id());
                return Err(e);
            }
        }
        info!(
            "[BlockProcess] executed transactions: {}, block id: {:?}",
            executed_block.block.transactions().len(),
            executed_block.block.id()
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

        ctx.put_shared::<crossbeam::channel::Receiver<Arc<ExecutedBlockInfo>>>(
            self.receiver.clone(),
        )?;

        ctx.run_interval(
            std::time::Duration::from_millis(10),
            |ctx: &mut ServiceContext<'_, Self>| {
                let receiver = ctx
                    .get_shared::<crossbeam::channel::Receiver<Arc<ExecutedBlockInfo>>>()
                    .expect("get receiver error");

                while let std::result::Result::Ok(executed_block_info) = receiver.try_recv() {
                    match &executed_block_info.from {
                        ExecuteBlockFrom::LocalMinedBlock => {
                            let bus = ctx.bus_ref().clone();
                            let _ = bus.broadcast(NewDagBlock {
                                executed_block: Arc::new(
                                    executed_block_info.executed_block.clone(),
                                ),
                            });
                        }
                        ExecuteBlockFrom::PeerMinedBlock(peer_id) => {
                            ctx.broadcast(NewDagBlockFromPeer {
                                executed_block: Arc::new(
                                    executed_block_info.executed_block.header().clone(),
                                ),
                            });
                            let bus = ctx.bus_ref().clone();
                            let _ = bus.broadcast(SelectHeaderState::new(
                                peer_id.clone(),
                                executed_block_info.executed_block.block().clone(),
                            ));
                        }
                    }
                }
            },
        );

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

        let time_service = self.time_service.clone();
        let storage = self.storage.clone();
        let dag = self.dag.clone();
        let sender = self.sender.clone();

        async_std::task::spawn(async move {
            match Self::execute(msg.get_block().clone(), time_service, storage, dag) {
                std::result::Result::Ok(executed_block) => {
                    match sender.send(Arc::new(ExecutedBlockInfo {
                        executed_block,
                        from: ExecuteBlockFrom::PeerMinedBlock(msg.get_peer_id()),
                    })) {
                        std::result::Result::Ok(_) => (),
                        Err(e) => error!(
                            "execute a peer block {:?} error: {}",
                            msg.get_block().id(),
                            e
                        ),
                    }
                }
                Err(e) => {
                    error!(
                        "execute a peer block {:?} error: {:?}",
                        msg.get_block().id(),
                        e
                    );
                }
            }
        });
    }
}

impl EventHandler<Self, MinedBlock> for ExecuteService {
    fn handle_event(&mut self, msg: MinedBlock, _ctx: &mut ServiceContext<Self>) {
        let MinedBlock(new_block) = msg;
        let id = new_block.header().id();
        debug!("try connect mined block: {}", id);

        let time_service = self.time_service.clone();
        let storage = self.storage.clone();
        let dag = self.dag.clone();
        let sender = self.sender.clone();
        let block = new_block.as_ref().clone();
        let block_id = block.id();

        async_std::task::spawn(async move {
            match Self::execute(block, time_service, storage, dag) {
                std::result::Result::Ok(executed_block) => {
                    match sender.send(Arc::new(ExecutedBlockInfo {
                        executed_block,
                        from: ExecuteBlockFrom::LocalMinedBlock,
                    })) {
                        std::result::Result::Ok(_) => (),
                        Err(e) => error!("execute a local block {:?} error: {}", block_id, e),
                    }
                }
                Err(e) => {
                    error!("execute a local block {:?} error: {:?}", block_id, e);
                }
            }
        });
    }
}
