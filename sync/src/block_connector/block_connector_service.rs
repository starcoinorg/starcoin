// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
use super::CheckBlockConnectorHashValue;
#[cfg(test)]
use super::CreateBlockRequest;
#[cfg(test)]
use super::CreateBlockResponse;
use super::ResetRequest;
use crate::block_connector::{ExecuteRequest, WriteBlockChainService};
use crate::sync::{CheckSyncEvent, SyncService};
use crate::tasks::{BlockConnectedEvent, BlockConnectedFinishEvent, BlockDiskCheckEvent};
#[cfg(test)]
use anyhow::bail;
use anyhow::{format_err, Ok, Result};
use network_api::PeerProvider;
use starcoin_chain::verifier::FullVerifier;
use starcoin_chain::BlockChain;
use starcoin_chain::ChainWriter;
use starcoin_chain_api::{ChainReader, ConnectBlockError, WriteableChainService};
use starcoin_config::TimeService;
use starcoin_config::{NodeConfig, G_CRATE_VERSION};
use starcoin_dag::blockdag::BlockDAG;
use starcoin_dag::service::pruning_point_service::PruningPointInfoChannel;
use starcoin_dag::service::pruning_point_service::PruningPointMessage;
use starcoin_executor::VMMetrics;
use starcoin_logger::prelude::*;
use starcoin_network::NetworkServiceRef;
use starcoin_service_registry::bus::Bus;
use starcoin_service_registry::{
    ActorService, EventHandler, ServiceContext, ServiceFactory, ServiceHandler,
};
use starcoin_storage::block_info::BlockInfoStore;
use starcoin_storage::{BlockStore, Storage};
use starcoin_sync_api::PeerNewBlock;
use starcoin_txpool::TxPoolService;
use starcoin_txpool_api::TxPoolSyncService;
#[cfg(test)]
use starcoin_txpool_mock_service::MockTxPoolService;
use starcoin_types::block::ExecutedBlock;
use starcoin_types::sync_status::SyncStatus;
use starcoin_types::system_events::NewDagBlock;
use starcoin_types::system_events::NewDagBlockFromPeer;
use starcoin_types::system_events::NewHeadBlock;
use starcoin_types::system_events::{MinedBlock, SyncStatusChangeEvent, SystemShutdown};
use std::sync::Arc;
use sysinfo::{DiskExt, System, SystemExt};
const DISK_CHECKPOINT_FOR_PANIC: u64 = 1024 * 1024 * 1024 * 3;
const DISK_CHECKPOINT_FOR_WARN: u64 = 1024 * 1024 * 1024 * 5;

pub struct BlockConnectorService<TransactionPoolServiceT>
where
    TransactionPoolServiceT: TxPoolSyncService + 'static,
{
    chain_service: WriteBlockChainService<TransactionPoolServiceT>,
    sync_status: Option<SyncStatus>,
    config: Arc<NodeConfig>,
    pruning_point_channel: PruningPointInfoChannel,
    storage: Arc<Storage>,
    dag: BlockDAG,
    time_service: Arc<dyn TimeService>,
}

impl<TransactionPoolServiceT> BlockConnectorService<TransactionPoolServiceT>
where
    TransactionPoolServiceT: TxPoolSyncService + 'static,
{
    pub fn new(
        chain_service: WriteBlockChainService<TransactionPoolServiceT>,
        config: Arc<NodeConfig>,
        pruning_point_channel: PruningPointInfoChannel,
        storage: Arc<Storage>,
        dag: BlockDAG,
        time_service: Arc<dyn TimeService>,
    ) -> Self {
        Self {
            chain_service,
            sync_status: None,
            config,
            pruning_point_channel,
            storage,
            dag,
            time_service,
        }
    }

    pub fn is_synced(&self) -> bool {
        match self.sync_status.as_ref() {
            Some(sync_status) => sync_status.is_nearly_synced(),
            None => false,
        }
    }

    pub fn check_disk_space(&mut self) -> Option<Result<u64>> {
        if System::IS_SUPPORTED {
            let mut sys = System::new_all();
            if sys.disks().len() == 1 {
                let disk = &sys.disks()[0];
                if DISK_CHECKPOINT_FOR_PANIC > disk.available_space() {
                    return Some(Err(anyhow::anyhow!(
                        "Disk space is less than {} GB, please add disk space.",
                        DISK_CHECKPOINT_FOR_PANIC / 1024 / 1024 / 1024
                    )));
                } else if DISK_CHECKPOINT_FOR_WARN > disk.available_space() {
                    return Some(Ok(disk.available_space() / 1024 / 1024 / 1024));
                }
            } else {
                sys.sort_disks_by(|a, b| {
                    if a.mount_point()
                        .starts_with(b.mount_point().to_str().unwrap())
                    {
                        std::cmp::Ordering::Less
                    } else {
                        std::cmp::Ordering::Greater
                    }
                });

                let base_data_dir = self.config.base().base_data_dir.path();
                for disk in sys.disks() {
                    if base_data_dir.starts_with(disk.mount_point()) {
                        if DISK_CHECKPOINT_FOR_PANIC > disk.available_space() {
                            return Some(Err(anyhow::anyhow!(
                                "Disk space is less than {} GB, please add disk space.",
                                DISK_CHECKPOINT_FOR_PANIC / 1024 / 1024 / 1024
                            )));
                        } else if DISK_CHECKPOINT_FOR_WARN > disk.available_space() {
                            return Some(Ok(disk.available_space() / 1024 / 1024 / 1024));
                        }

                        break;
                    }
                }
            }
        }

        None
    }
}

impl<TransactionPoolServiceT> ServiceFactory<Self>
    for BlockConnectorService<TransactionPoolServiceT>
where
    TransactionPoolServiceT: TxPoolSyncService + 'static,
{
    fn create(ctx: &mut ServiceContext<Self>) -> Result<Self> {
        let config = ctx.get_shared::<Arc<NodeConfig>>()?;
        let bus = ctx.bus_ref().clone();
        let txpool = ctx.get_shared::<TransactionPoolServiceT>()?;
        let storage = ctx.get_shared::<Arc<Storage>>()?;
        let startup_info = storage
            .get_startup_info()?
            .ok_or_else(|| format_err!("Startup info should exist."))?;
        let vm_metrics = ctx.get_shared_opt::<VMMetrics>()?;
        let dag = ctx.get_shared::<BlockDAG>()?;
        let pruning_point_channel = ctx.get_shared::<PruningPointInfoChannel>()?;
        let chain_service = WriteBlockChainService::new(
            config.clone(),
            startup_info,
            storage.clone(),
            txpool,
            bus,
            vm_metrics,
            dag.clone(),
        )?;
        let time_service = config.net().time_service();

        Ok(Self::new(
            chain_service,
            config,
            pruning_point_channel,
            storage,
            dag,
            time_service,
        ))
    }
}

impl<TransactionPoolServiceT> ActorService for BlockConnectorService<TransactionPoolServiceT>
where
    TransactionPoolServiceT: TxPoolSyncService + 'static,
{
    fn started(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        //TODO figure out a more suitable value.
        let merge_depth = self
            .chain_service
            .get_dag()
            .block_depth_manager()
            .merge_depth()
            .saturating_mul(3);
        ctx.set_mailbox_capacity(merge_depth as usize);
        ctx.subscribe::<SyncStatusChangeEvent>();
        ctx.subscribe::<MinedBlock>();
        ctx.subscribe::<NewDagBlock>();

        ctx.run_interval(std::time::Duration::from_secs(3), move |ctx| {
            ctx.notify(crate::tasks::BlockDiskCheckEvent {});
        });

        Ok(())
    }

    fn stopped(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.unsubscribe::<SyncStatusChangeEvent>();
        ctx.unsubscribe::<MinedBlock>();
        ctx.unsubscribe::<NewDagBlock>();
        Ok(())
    }
}

impl<TransactionPoolServiceT> EventHandler<Self, BlockDiskCheckEvent>
    for BlockConnectorService<TransactionPoolServiceT>
where
    TransactionPoolServiceT: TxPoolSyncService + 'static,
{
    fn handle_event(&mut self, _: BlockDiskCheckEvent, ctx: &mut ServiceContext<Self>) {
        if let Some(res) = self.check_disk_space() {
            match res {
                std::result::Result::Ok(available_space) => {
                    warn!("Available diskspace only {}/GB left ", available_space)
                }
                Err(e) => {
                    error!("{}", e);
                    ctx.broadcast(SystemShutdown);
                }
            }
        }
    }
}

impl<TransactionPoolServiceT> EventHandler<Self, NewDagBlock>
    for BlockConnectorService<TransactionPoolServiceT>
where
    TransactionPoolServiceT: TxPoolSyncService + 'static,
{
    fn handle_event(&mut self, msg: NewDagBlock, ctx: &mut ServiceContext<Self>) {
        info!("jacktest: NewDagBlock in block connector, start");
        let block_header = match self.chain_service.switch_header(&msg.executed_block, ctx) {
            std::result::Result::Ok(block_header) => block_header,
            Err(e) => {
                error!(
                    "failed to switch header when processing NewDagBlock, error: {:?}, id: {:?}",
                    e,
                    msg.executed_block.header().id()
                );
                return;
            }
        };

        let _consume = self
            .pruning_point_channel
            .pruning_receiver
            .try_iter()
            .count();
        match self
            .pruning_point_channel
            .pruning_sender
            .send(PruningPointMessage { block_header })
        {
            std::result::Result::Ok(_) => (),
            Err(e) => {
                error!(
                    "failed to send NewDagBlock for calculating the pruning point, error: {:?}",
                    e
                );
            }
        }
    }
}

impl EventHandler<Self, BlockConnectedEvent> for BlockConnectorService<TxPoolService> {
    fn handle_event(&mut self, msg: BlockConnectedEvent, ctx: &mut ServiceContext<Self>) {
        //because this block has execute at sync task, so just try connect to select head chain.
        //TODO refactor connect and execute
        let block = msg.block;
        let feedback = msg.feedback;

        match msg.action {
            crate::tasks::BlockConnectAction::ConnectNewBlock => {
                let current_block_info = self
                    .storage
                    .get_block_info(self.chain_service.get_main_header().id())
                    .expect("block info not found in new dag block message")
                    .expect("block info not found in new dag block message");
                if let Err(e) = self.chain_service.try_connect(block.clone()) {
                    error!("Process connected new block from sync error: {:?}", e);
                } else {
                    let block_info = self
                        .storage
                        .get_block_info(block.id())
                        .expect("block info not found in new dag block message")
                        .expect("block info not found in new dag block message");
                    if current_block_info.total_difficulty < block_info.total_difficulty {
                        ctx.broadcast(NewDagBlock {
                            executed_block: Arc::new(ExecutedBlock::new(block, block_info)),
                        });
                    }
                }
            }
            crate::tasks::BlockConnectAction::ConnectExecutedBlock => {
                if let Err(e) = self.chain_service.switch_new_main(block.header().id(), ctx) {
                    error!("Process connected executed block from sync error: {:?}", e);
                }
            }
        }

        feedback.map(|f| f.unbounded_send(BlockConnectedFinishEvent));
    }
}

#[cfg(test)]
impl EventHandler<Self, BlockConnectedEvent> for BlockConnectorService<MockTxPoolService> {
    fn handle_event(&mut self, msg: BlockConnectedEvent, ctx: &mut ServiceContext<Self>) {
        //because this block has execute at sync task, so just try connect to select head chain.
        //TODO refactor connect and execute
        let block = msg.block;
        let feedback = msg.feedback;

        match msg.action {
            crate::tasks::BlockConnectAction::ConnectNewBlock => {
                if let Err(e) = self.chain_service.apply_failed(block) {
                    error!("Process connected new block from sync error: {:?}", e);
                }
            }
            crate::tasks::BlockConnectAction::ConnectExecutedBlock => {
                if let Err(e) = self.chain_service.switch_new_main(block.header().id(), ctx) {
                    error!("Process connected executed block from sync error: {:?}", e);
                }
            }
        }

        feedback.map(|f| f.unbounded_send(BlockConnectedFinishEvent));
    }
}

impl<TransactionPoolServiceT> EventHandler<Self, MinedBlock>
    for BlockConnectorService<TransactionPoolServiceT>
where
    TransactionPoolServiceT: TxPoolSyncService + 'static,
{
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

impl<TransactionPoolServiceT> EventHandler<Self, SyncStatusChangeEvent>
    for BlockConnectorService<TransactionPoolServiceT>
where
    TransactionPoolServiceT: TxPoolSyncService + 'static,
{
    fn handle_event(&mut self, msg: SyncStatusChangeEvent, _ctx: &mut ServiceContext<Self>) {
        self.sync_status = Some(msg.0);
    }
}

impl<TransactionPoolServiceT> EventHandler<Self, PeerNewBlock>
    for BlockConnectorService<TransactionPoolServiceT>
where
    TransactionPoolServiceT: TxPoolSyncService + 'static,
{
    fn handle_event(&mut self, msg: PeerNewBlock, ctx: &mut ServiceContext<Self>) {
        if !self.is_synced() {
            debug!("[connector] Ignore PeerNewBlock event because the node has not been synchronized yet.");
            return;
        }
        let peer_id = msg.get_peer_id();
        let current_block_info = self
            .storage
            .get_block_info(self.chain_service.get_main_header().id())
            .expect("block info not found in new dag block message")
            .expect("block info not found in new dag block message");
        if let Err(e) = self.chain_service.try_connect(msg.get_block().clone()) {
            match e.downcast::<ConnectBlockError>() {
                std::result::Result::Ok(connect_error) => {
                    match connect_error {
                        ConnectBlockError::FutureBlock(block) => {
                            //TODO cache future block
                            if let std::result::Result::Ok(sync_service) =
                                ctx.service_ref::<SyncService>()
                            {
                                info!(
                                    "BlockConnector try connect future block ({:?},{}), peer_id:{:?}, notify Sync service check sync.",
                                    block.id(),
                                    block.header().number(),
                                    peer_id
                                );
                                let _ = sync_service.notify(CheckSyncEvent::default());
                            }
                        }
                        e => {
                            warn!("BlockConnector fail: {:?}, peer_id:{:?}", e, peer_id);
                            if let Err(err) = self.storage.save_failed_block(
                                msg.get_block().id(),
                                msg.get_block().clone(),
                                Some(peer_id.clone()),
                                format!("{:?}", e),
                                G_CRATE_VERSION.to_string(),
                            ) {
                                warn!(
                                    "Save FailedBlock err: {:?}, block_id:{:?}.",
                                    err,
                                    msg.get_block().id()
                                );
                            }

                            if let Err(e1) = ctx
                                .get_shared::<NetworkServiceRef>()
                                .map(|network| network.report_peer(peer_id, e.reputation()))
                            {
                                warn!("Get NetworkServiceRef err: {:?}.", e1);
                            }
                        }
                    }
                }
                Err(e) => warn!("BlockConnector fail: {:?}, peer_id:{:?}", e, peer_id),
            }
        } else {
            ctx.broadcast(NewDagBlockFromPeer {
                executed_block: Arc::new(msg.get_block().header().clone()),
            });
            let block_info = self
                .storage
                .get_block_info(msg.get_block().id())
                .expect("block info not found in new dag block message")
                .expect("block info not found in new dag block message");
            if current_block_info.total_difficulty < block_info.total_difficulty {
                self.config
                    .net()
                    .time_service()
                    .adjust(msg.get_block().header().timestamp());
                ctx.broadcast(NewHeadBlock {
                    executed_block: Arc::new(ExecutedBlock {
                        block: msg.get_block().clone(),
                        block_info,
                    }),
                });
            }
        }
    }
}

impl<TransactionPoolServiceT> ServiceHandler<Self, ResetRequest>
    for BlockConnectorService<TransactionPoolServiceT>
where
    TransactionPoolServiceT: TxPoolSyncService + 'static,
{
    fn handle(&mut self, msg: ResetRequest, _ctx: &mut ServiceContext<Self>) -> Result<()> {
        self.chain_service.reset(msg.block_hash)
    }
}

impl<TransactionPoolServiceT> ServiceHandler<Self, ExecuteRequest>
    for BlockConnectorService<TransactionPoolServiceT>
where
    TransactionPoolServiceT: TxPoolSyncService + 'static,
{
    fn handle(
        &mut self,
        msg: ExecuteRequest,
        _ctx: &mut ServiceContext<Self>,
    ) -> Result<ExecutedBlock> {
        self.chain_service.execute(msg.block)
    }
}

#[cfg(test)]
impl<TransactionPoolServiceT> ServiceHandler<Self, CreateBlockRequest>
    for BlockConnectorService<TransactionPoolServiceT>
where
    TransactionPoolServiceT: TxPoolSyncService + 'static,
{
    fn handle(
        &mut self,
        msg: CreateBlockRequest,
        ctx: &mut ServiceContext<Self>,
    ) -> <CreateBlockRequest as starcoin_service_registry::ServiceRequest>::Response {
        for _i in 0..msg.count {
            let block = self.chain_service.create_block(
                msg.author,
                msg.parent_hash,
                msg.user_txns.clone(),
                msg.uncles.clone(),
                msg.block_gas_limit,
                msg.tips.clone(),
            )?;
            let current_block_info = self
                .storage
                .get_block_info(self.chain_service.get_main_header().id())?
                .ok_or_else(|| anyhow::anyhow!("block info not found"))?;
            self.chain_service.try_connect(block.clone())?;
            let block_info = self
                .storage
                .get_block_info(block.id())?
                .ok_or_else(|| anyhow::anyhow!("block info not found"))?;
            if current_block_info.total_difficulty < block_info.total_difficulty {
                ctx.broadcast(NewHeadBlock {
                    executed_block: Arc::new(ExecutedBlock { block, block_info }),
                });
            }
        }
        Ok(CreateBlockResponse)
    }
}

#[cfg(test)]
impl<TransactionPoolServiceT> ServiceHandler<Self, CheckBlockConnectorHashValue>
    for BlockConnectorService<TransactionPoolServiceT>
where
    TransactionPoolServiceT: TxPoolSyncService + 'static,
{
    fn handle(
        &mut self,
        msg: CheckBlockConnectorHashValue,
        _ctx: &mut ServiceContext<Self>,
    ) -> Result<()> {
        if self.chain_service.get_main_header().id() == msg.head_hash {
            info!("the branch in chain service is the same as target's branch");
            Ok(())
        } else {
            info!("mock branch in chain service is not the same as target's branch");
            bail!("blockchain in chain service is not the same as target!");
        }
    }
}
