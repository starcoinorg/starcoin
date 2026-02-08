use std::sync::Arc;
use std::time::Duration;

use anyhow::{Ok, Result};
use network_api::PeerId;
use sp_utils::thread_pool::RAYON_EXEC_POOL;
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
use starcoin_storage::Storage2;
use starcoin_storage::{BlockStore, Storage};
use starcoin_sync_api::{PeerNewBlock, SelectHeaderState};
use starcoin_types::block::Block;
use starcoin_types::sync_status::SyncStatus;
use starcoin_types::system_events::{
    MinedBlock, NewDagBlock, NewDagBlockFromPeer, SyncStatusChangeEvent,
};

use crate::sync::CheckSyncEvent;

#[derive(Debug, Clone)]
enum ExecuteResult {
    Executed(Box<ExecutedBlock>),
    AlreadyExecuted,
    TryLater,
}

#[derive(Debug, Clone)]
enum ExecuteBlockFrom {
    LocalMinedBlock(HashValue),
    PeerMinedBlock(HashValue, PeerId),
}

#[derive(Debug, Clone)]
struct ExecutedBlockInfo {
    executed_block: Option<Box<ExecutedBlock>>,
    from: ExecuteBlockFrom,
}

pub struct ExecuteService {
    time_service: Arc<dyn TimeService>,
    storage: Arc<Storage>,
    storage2: Arc<Storage2>,
    dag: BlockDAG,
    sync_status: Option<SyncStatus>,
}

const PEER_BLOCK_RETRY_DELAY_MS: u64 = 200;

#[derive(Debug, Clone)]
struct PeerBlockTryLater {
    peer_id: PeerId,
    block: Block,
}

impl ExecuteService {
    fn new(
        time_service: Arc<dyn TimeService>,
        storage: Arc<Storage>,
        storage2: Arc<Storage2>,
        dag: BlockDAG,
    ) -> Self {
        Self {
            time_service,
            storage,
            storage2,
            dag,
            sync_status: None,
        }
    }

    fn is_synced(&self) -> bool {
        match self.sync_status.as_ref() {
            Some(sync_status) => sync_status.is_nearly_synced(),
            None => false,
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

    fn has_dag_block(block_id: HashValue, storage: Arc<Storage>, dag: BlockDAG) -> Result<bool> {
        let header = match storage.get_block_header_by_hash(block_id)? {
            Some(header) => header,
            None => return Ok(false),
        };

        if storage.get_block_info(header.id())?.is_none() {
            return Ok(false);
        }

        dag.has_block_connected(&header)
    }

    fn execute(
        new_block: Block,
        time_service: Arc<dyn TimeService>,
        storage: Arc<Storage>,
        storage2: Arc<Storage2>,
        dag: BlockDAG,
    ) -> Result<ExecuteResult> {
        if Self::has_dag_block(new_block.id(), storage.clone(), dag.clone())? {
            info!(
                "skip duplicated dag block execution, block id: {:?}",
                new_block.id()
            );
            return Ok(ExecuteResult::AlreadyExecuted);
        }

        for parent_id in new_block.header().parents_hash() {
            if !Self::check_parent_ready(*parent_id, storage.clone(), dag.clone())? {
                return Ok(ExecuteResult::TryLater);
            }
        }

        let mut chain = match BlockChain::new(
            time_service.clone(),
            new_block.header().parent_hash(),
            storage.clone(),
            storage2.clone(),
            None,
            dag.clone(),
        ) {
            std::result::Result::Ok(chain) => chain,
            Err(e) => {
                error!(
                    "new block chain error when processing the mined block: {:?}",
                    e
                );
                return Err(e);
            }
        };

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

        match chain.connect(executed_block.clone()) {
            std::result::Result::Ok(_) => (),
            Err(e) => {
                error!("when connecting the mined block, failed to connect block error: {:?}, id: {:?}", e, executed_block.block().id());
                return Err(e);
            }
        }
        info!(
            "[BlockProcess] executed transactions: {}, transactions2: {}, block id: {:?}",
            executed_block.block().transactions().len(),
            executed_block.block().transactions2().len(),
            executed_block.block().id()
        );
        Ok(ExecuteResult::Executed(Box::new(executed_block)))
    }
}

impl ServiceFactory<Self> for ExecuteService {
    fn create(ctx: &mut ServiceContext<Self>) -> Result<Self> {
        let config = ctx.get_shared::<Arc<NodeConfig>>()?;
        let time_service = config.net().time_service();
        let storage = ctx.get_shared::<Arc<Storage>>()?;
        let storage2 = ctx.get_shared::<Arc<Storage2>>()?;
        let dag = ctx.get_shared::<BlockDAG>()?;

        let mut service = Self::new(time_service, storage, storage2, dag);
        if let Ok(Some(sync_status)) = ctx.get_shared_opt::<SyncStatus>() {
            service.sync_status = Some(sync_status);
        }
        Ok(service)
    }
}

impl ActorService for ExecuteService {
    fn started(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.subscribe::<MinedBlock>();
        ctx.subscribe::<PeerNewBlock>();
        ctx.subscribe::<ExecutedBlockInfo>();
        ctx.subscribe::<SyncStatusChangeEvent>();

        Ok(())
    }

    fn stopped(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.unsubscribe::<MinedBlock>();
        ctx.unsubscribe::<PeerNewBlock>();
        ctx.unsubscribe::<ExecutedBlockInfo>();
        ctx.unsubscribe::<SyncStatusChangeEvent>();

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
                            executed_block: Arc::new(*executed_block.clone()),
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
                            executed_block: Arc::new(executed_block.block().header().clone()),
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

impl EventHandler<Self, SyncStatusChangeEvent> for ExecuteService {
    fn handle_event(&mut self, msg: SyncStatusChangeEvent, _ctx: &mut ServiceContext<Self>) {
        self.sync_status = Some(msg.0);
    }
}

impl EventHandler<Self, PeerBlockTryLater> for ExecuteService {
    fn handle_event(&mut self, msg: PeerBlockTryLater, ctx: &mut ServiceContext<Self>) {
        let PeerBlockTryLater { peer_id, block } = msg;
        ctx.run_later(
            Duration::from_millis(PEER_BLOCK_RETRY_DELAY_MS),
            move |service_ctx| {
                if let Err(e) = service_ctx
                    .self_ref()
                    .notify(PeerNewBlock::new(peer_id.clone(), block.clone()))
                {
                    error!("retry notify peer block error: {:?}", e);
                }
            },
        );
    }
}

impl EventHandler<Self, PeerNewBlock> for ExecuteService {
    fn handle_event(&mut self, msg: PeerNewBlock, ctx: &mut ServiceContext<Self>) {
        if !self.is_synced() {
            debug!(
                "[execute] Ignore PeerNewBlock event because the node has not been synchronized yet."
            );
            return;
        }

        let time_service = self.time_service.clone();
        let storage = self.storage.clone();
        let storage2 = self.storage2.clone();
        let dag = self.dag.clone();
        let self_ref = ctx.self_ref();

        RAYON_EXEC_POOL.spawn(move || {
            match Self::execute(
                msg.get_block().clone(),
                time_service,
                storage,
                storage2,
                dag,
            ) {
                std::result::Result::Ok(execute_result) => {
                    if let Err(e) = match execute_result {
                        ExecuteResult::Executed(executed_block) => self_ref
                            .notify(ExecutedBlockInfo {
                                executed_block: Some(executed_block),
                                from: ExecuteBlockFrom::PeerMinedBlock(
                                    msg.get_block().id(),
                                    msg.get_peer_id(),
                                ),
                            })
                            .map_err(anyhow::Error::from),
                        ExecuteResult::AlreadyExecuted => Ok(()),
                        ExecuteResult::TryLater => self_ref
                            .notify(PeerBlockTryLater {
                                peer_id: msg.get_peer_id(),
                                block: msg.get_block().clone(),
                            })
                            .map_err(anyhow::Error::from),
                    } {
                        error!(
                            "execute a peer block {:?} error: {:?}",
                            msg.get_block().id(),
                            e
                        );
                    }
                }
                Err(e) => {
                    error!(
                        "execute a peer block {:?} error: {:?}",
                        msg.get_block().id(),
                        e
                    );
                    if let Err(e) = self_ref.notify(ExecutedBlockInfo {
                        executed_block: None, // force to star sync
                        from: ExecuteBlockFrom::PeerMinedBlock(
                            msg.get_block().id(),
                            msg.get_peer_id(),
                        ),
                    }) {
                        error!("notify error: {:?}", e);
                    }
                }
            }
        });
    }
}

impl EventHandler<Self, MinedBlock> for ExecuteService {
    fn handle_event(&mut self, msg: MinedBlock, ctx: &mut ServiceContext<Self>) {
        let MinedBlock(new_block) = msg;
        let id = new_block.header().id();
        debug!("try connect mined block: {}", id);

        let time_service = self.time_service.clone();
        let storage = self.storage.clone();
        let storage2 = self.storage2.clone();
        let dag = self.dag.clone();
        let block = new_block.as_ref().clone();
        let block_id = block.id();
        let self_ref = ctx.self_ref();

        RAYON_EXEC_POOL.spawn(move || {
            match Self::execute(block, time_service, storage, storage2, dag) {
                std::result::Result::Ok(executed_result) => {
                    if let Err(e) = match executed_result {
                        ExecuteResult::Executed(executed_block) => self_ref
                            .notify(ExecutedBlockInfo {
                                executed_block: Some(executed_block),
                                from: ExecuteBlockFrom::LocalMinedBlock(block_id),
                            })
                            .map_err(anyhow::Error::from),
                        ExecuteResult::AlreadyExecuted => Ok(()),
                        ExecuteResult::TryLater => self_ref
                            .notify(MinedBlock(new_block))
                            .map_err(anyhow::Error::from),
                    } {
                        error!("execute a local block {:?} error: {:?}", block_id, e);
                    }
                }
                Err(e) => {
                    error!("execute a local block {:?} error: {:?}", block_id, e);
                }
            }
        });
    }
}
