use std::sync::Arc;

use anyhow::{Ok, Result};
use futures::channel::mpsc;
use futures::SinkExt;
use network_api::PeerId;
use starcoin_chain::{verifier::FullVerifier, BlockChain};
use starcoin_chain::{ChainReader, ChainWriter};
use starcoin_chain_api::ExecutedBlock;
use starcoin_config::{NodeConfig, TimeService};
use starcoin_crypto::HashValue;
use starcoin_dag::blockdag::BlockDAG;
use starcoin_logger::prelude::{debug, error, info, warn};
use starcoin_service_registry::{
    bus::Bus, ActorService, EventHandler, ServiceContext, ServiceFactory,
};
use starcoin_storage::block_info::BlockInfoStore;
use starcoin_storage::{BlockStore, Storage};
use starcoin_sync_api::{PeerNewBlock, SelectHeaderState};
use starcoin_types::block::Block;
use starcoin_types::system_events::{MinedBlock, NewDagBlock, NewDagBlockFromPeer};

use crate::sync::CheckSyncEvent;

#[derive(Debug, Clone)]
enum ExecuteBlockFrom {
    LocalMinedBlock(HashValue),
    PeerMinedBlock(HashValue, PeerId),
}

#[derive(Debug, Clone)]
struct ExecutedBlockInfo {
    executed_block: Option<ExecutedBlock>,
    from: ExecuteBlockFrom,
}

pub struct ExecuteService {
    time_service: Arc<dyn TimeService>,
    storage: Arc<Storage>,
    dag: BlockDAG,
    sender: Option<mpsc::UnboundedSender<ExecutedBlockInfo>>,
}

impl ExecuteService {
    fn new(time_service: Arc<dyn TimeService>, storage: Arc<Storage>, dag: BlockDAG) -> Self {
        Self {
            time_service,
            storage,
            dag,
            sender: None,
        }
    }

    fn check_parent_ready(
        parent_id: HashValue,
        storage: Arc<Storage>,
        dag: BlockDAG,
    ) -> Result<bool> {
        let header = match storage.get_block_header_by_hash(parent_id)? {
            Some(header) => header,
            None => return Ok(false),
        };

        if storage.get_block_info(header.id())?.is_none() {
            return Ok(false);
        }

        dag.has_block_connected(&header)
    }

    async fn execute(
        new_block: Block,
        time_service: Arc<dyn TimeService>,
        storage: Arc<Storage>,
        dag: BlockDAG,
    ) -> Result<ExecutedBlock> {
        info!(
            "[BlockProcess] now start to execute the block and try to check the parents: {:?}",
            new_block.id()
        );

        let id = new_block.id();

        for parent_id in new_block.header().parents_hash() {
            let mut count: u64 = 3000;
            while !Self::check_parent_ready(parent_id, storage.clone(), dag.clone())? && count > 0 {
                async_std::task::sleep(std::time::Duration::from_millis(10)).await;
                count = count.saturating_sub(1);
                if count == 0 {
                    return Err(anyhow::anyhow!(
                        "wait dag block timeout, for block id: {:?}",
                        parent_id
                    ));
                }
            }
        }

        info!(
            "[BlockProcess] now create the block's selected parent chain object: {:?}",
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

        let (sender, receiver) = mpsc::unbounded::<ExecutedBlockInfo>();
        self.sender = Some(sender);
        ctx.add_stream(receiver);

        Ok(())
    }

    fn stopped(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.unsubscribe::<MinedBlock>();
        ctx.unsubscribe::<PeerNewBlock>();
        Ok(())
    }
}

impl EventHandler<Self, ExecutedBlockInfo> for ExecuteService {
    fn handle_event(
        &mut self,
        executed_block_info: ExecutedBlockInfo,
        ctx: &mut ServiceContext<Self>,
    ) {
        match &executed_block_info.from {
            ExecuteBlockFrom::LocalMinedBlock(block_id) => {
                let bus = ctx.bus_ref().clone();
                match &executed_block_info.executed_block {
                    Some(executed_block) => {
                        let _ = bus.broadcast(NewDagBlock {
                            executed_block: Arc::new(executed_block.clone()),
                        });
                    }
                    None => {
                        error!("failed to execute the mined block, id: {:?} ", block_id)
                    }
                }
            }
            ExecuteBlockFrom::PeerMinedBlock(block_id, peer_id) => {
                match &executed_block_info.executed_block {
                    Some(executed_block) => {
                        let bus = ctx.bus_ref().clone();
                        let _ = bus.broadcast(NewDagBlockFromPeer {
                            executed_block: Arc::new(executed_block.header().clone()),
                        });
                        let bus = ctx.bus_ref().clone();
                        let _ = bus.broadcast(SelectHeaderState::new(
                            peer_id.clone(),
                            executed_block.block().clone(),
                        ));
                    }
                    None => {
                        warn!("failed to execute the peer block, id: {:?} ", block_id);
                        ctx.broadcast(CheckSyncEvent::default());
                    }
                }
            }
        }
    }
}

impl EventHandler<Self, PeerNewBlock> for ExecuteService {
    fn handle_event(&mut self, msg: PeerNewBlock, _ctx: &mut ServiceContext<Self>) {
        let time_service = self.time_service.clone();
        let storage = self.storage.clone();
        let dag = self.dag.clone();
        let mut sender = self.sender.clone();

        async_std::task::spawn(async move {
            match Self::execute(msg.get_block().clone(), time_service, storage, dag).await {
                std::result::Result::Ok(executed_block) => {
                    match sender.as_mut().unwrap().start_send(ExecutedBlockInfo {
                        executed_block: Some(executed_block),
                        from: ExecuteBlockFrom::PeerMinedBlock(
                            msg.get_block().id(),
                            msg.get_peer_id(),
                        ),
                    }) {
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
                    match sender.as_mut().unwrap().start_send(ExecutedBlockInfo {
                        executed_block: None,
                        from: ExecuteBlockFrom::PeerMinedBlock(
                            msg.get_block().id(),
                            msg.get_peer_id(),
                        ),
                    }) {
                        std::result::Result::Ok(_) => (),
                        Err(e) => error!(
                            "execute a peer block {:?} error: {}",
                            msg.get_block().id(),
                            e
                        ),
                    }
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
        let mut sender = self.sender.clone();
        let block = new_block.as_ref().clone();
        let block_id = block.id();

        async_std::task::spawn(async move {
            match Self::execute(block, time_service, storage, dag).await {
                std::result::Result::Ok(executed_block) => {
                    match sender.as_mut().unwrap().start_send(ExecutedBlockInfo {
                        executed_block: Some(executed_block),
                        from: ExecuteBlockFrom::LocalMinedBlock(block_id),
                    }) {
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
