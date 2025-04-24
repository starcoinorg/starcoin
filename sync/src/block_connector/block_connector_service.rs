// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
use super::CheckBlockConnectorHashValue;
#[cfg(test)]
use super::CreateBlockRequest;
#[cfg(test)]
use super::CreateBlockResponse;
use crate::block_connector::{
    ExecuteRequest, MinerRequest, MinerResponse, ResetRequest, WriteBlockChainService,
};
use crate::sync::{CheckSyncEvent, SyncService};
use crate::tasks::{BlockConnectedEvent, BlockConnectedFinishEvent, BlockDiskCheckEvent};
use anyhow::{bail, format_err, Ok, Result};
use network_api::PeerProvider;
use starcoin_chain::BlockChain;
use starcoin_chain::ChainWriter;
use starcoin_chain_api::VerifiedBlock;
use starcoin_chain_api::{ChainReader, ConnectBlockError, WriteableChainService};
use starcoin_config::{NodeConfig, G_CRATE_VERSION};
use starcoin_consensus::Consensus;
use starcoin_crypto::HashValue;
use starcoin_dag::blockdag::BlockDAG;
use starcoin_dag::blockdag::MineNewDagBlockInfo;
use starcoin_dag::service::pruning_point_service::PruningPointInfoChannel;
use starcoin_dag::service::pruning_point_service::PruningPointMessage;
use starcoin_executor::VMMetrics;
use starcoin_genesis::Genesis;
use starcoin_logger::prelude::*;
use starcoin_network::NetworkServiceRef;
use starcoin_service_registry::bus::Bus;
use starcoin_service_registry::{
    ActorService, EventHandler, ServiceContext, ServiceFactory, ServiceHandler, ServiceRequest,
};
use starcoin_storage::{BlockStore, Storage};
use starcoin_sync_api::PeerNewBlock;
use starcoin_sync_api::SyncSpecificTargretRequest;
use starcoin_txpool::TxPoolService;
use starcoin_txpool_api::TxPoolSyncService;
#[cfg(test)]
use starcoin_txpool_mock_service::MockTxPoolService;
use starcoin_types::block::BlockHeader;
use starcoin_types::block::ExecutedBlock;
use starcoin_types::sync_status::SyncStatus;
use starcoin_types::system_events::NewDagBlock;
use starcoin_types::system_events::NewDagBlockFromPeer;
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
    storage: Arc<Storage>,
    vm_metrics: Option<VMMetrics>,
    genesis: Genesis,
    pruning_point_channel: PruningPointInfoChannel,
}

impl<TransactionPoolServiceT> BlockConnectorService<TransactionPoolServiceT>
where
    TransactionPoolServiceT: TxPoolSyncService + 'static,
{
    pub fn new(
        chain_service: WriteBlockChainService<TransactionPoolServiceT>,
        config: Arc<NodeConfig>,
        storage: Arc<Storage>,
        vm_metrics: Option<VMMetrics>,
        genesis: Genesis,
        pruning_point_channel: PruningPointInfoChannel,
    ) -> Self {
        Self {
            chain_service,
            sync_status: None,
            config,
            storage,
            vm_metrics,
            genesis,
            pruning_point_channel,
        }
    }

    pub fn is_synced(&self) -> bool {
        match self.sync_status.as_ref() {
            Some(sync_status) => sync_status.is_synced(),
            None => false,
        }
    }

    pub fn chain_head_id(&self) -> HashValue {
        self.chain_service.get_main().status().head.id()
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

    // return false if the number of the block is larger than the current number of the chain.
    // or return false if the gap of those two blocks is larger than 2 * G_BASE_MAX_UNCLES_PER_BLOCK
    // else return true.
    // return false will trigger the burden sync operation.
    // return true will trigger the specific(light) sync operation.
    fn is_near_block(&self, block_header: &BlockHeader) -> bool {
        let current_number = self.chain_service.get_main().status().head().number();
        if current_number <= block_header.number() {
            return false;
        }
        let gap = current_number.saturating_sub(block_header.number());
        let k = self.chain_service.get_dag().ghost_dag_manager().k() as u64;
        let config_gap = self
            .config
            .sync
            .lightweight_sync_max_gap()
            .map_or(k.saturating_mul(k), |max_gap| max_gap);
        debug!(
            "is-near-block: current_number: {:?}, block_number: {:?}, gap: {:?}, config_gap: {:?}",
            current_number,
            block_header.number(),
            gap,
            config_gap
        );
        gap <= config_gap
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
        let genesis = ctx.get_shared::<Genesis>()?;
        let pruning_point_channel = ctx.get_shared::<PruningPointInfoChannel>()?;
        let chain_service = WriteBlockChainService::new(
            config.clone(),
            startup_info,
            storage.clone(),
            txpool,
            bus,
            vm_metrics.clone(),
            dag,
        )?;

        Ok(Self::new(
            chain_service,
            config,
            storage,
            vm_metrics,
            genesis,
            pruning_point_channel,
        ))
    }
}

impl<TransactionPoolServiceT> ActorService for BlockConnectorService<TransactionPoolServiceT>
where
    TransactionPoolServiceT: TxPoolSyncService + 'static,
{
    fn started(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        //TODO figure out a more suitable value.
        ctx.set_mailbox_capacity(1024);
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
    fn handle_event(&mut self, msg: NewDagBlock, _ctx: &mut ServiceContext<Self>) {
        info!("NewDagBlock in block connect service");
        let chain = self
            .chain_service
            .get_main()
            .fork(self.chain_service.get_main().status().head().id())
            .unwrap_or_else(|e| {
                panic!(
                    "fork error when handle NewDagBlock in block connect service: {:?}",
                    e
                )
            });
        let new_chain = chain
            .selecte_dag_state(msg.executed_block.as_ref().clone())
            .unwrap_or_else(|e| {
                panic!(
                    "select dag state error when handle NewDagBlock in block connect service: {:?}",
                    e
                )
            });
        let block_header = new_chain.head_block().block.header().clone();
        self.chain_service.switch_header(new_chain);

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
                if let Err(e) = self.chain_service.try_connect(block) {
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

        let main = self.chain_service.get_main();
        let mut chain = BlockChain::new(
            main.time_service(),
            new_block.header().parent_hash(),
            main.get_storage(),
            None,
            main.dag(),
        )
        .unwrap_or_else(|e| {
            panic!(
                "new block chain error when processing the mined block: {:?}",
                e
            )
        });
        let bus = self.chain_service.get_bus();

        ctx.spawn(async move {
            let executed_block = chain
                .execute(VerifiedBlock {
                    block: new_block.as_ref().clone(),
                    ghostdata: None,
                })
                .unwrap();
            chain.connect(executed_block.clone()).unwrap();
            let _ = bus.broadcast(NewDagBlock {
                executed_block: Arc::new(executed_block),
            });
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
                                if !self.is_near_block(block.as_ref().header()) {
                                    let _ = sync_service.notify(CheckSyncEvent::default());
                                } else {
                                    let _ = sync_service.notify(SyncSpecificTargretRequest {
                                        block: Some(block.as_ref().clone()),
                                        block_id: block.id(),
                                        peer_id: Some(peer_id),
                                    });
                                }
                            }
                        }
                        e => {
                            warn!("BlockConnector fail: {:?}, peer_id:{:?}", e, peer_id);
                            if let Err(err) = self
                                .chain_service
                                .get_main()
                                .get_storage()
                                .save_failed_block(
                                    msg.get_block().id(),
                                    msg.get_block().clone(),
                                    Some(peer_id.clone()),
                                    format!("{:?}", e),
                                    G_CRATE_VERSION.to_string(),
                                )
                            {
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
            ctx.broadcast(NewDagBlockFromPeer);
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

impl<T> ServiceHandler<Self, MinerRequest> for BlockConnectorService<T>
where
    T: TxPoolSyncService + 'static,
{
    fn handle(
        &mut self,
        _msg: MinerRequest,
        _ctx: &mut ServiceContext<Self>,
    ) -> <MinerRequest as ServiceRequest>::Response {
        let main_header = self.chain_service.get_main().status().head().clone();
        let dag = self.chain_service.get_dag();
        let genesis = self.genesis.clone();
        let MineNewDagBlockInfo {
            tips,
            blue_blocks,
            pruning_point,
        } = if main_header.number() >= self.chain_service.get_main().get_pruning_height() {
            let pruning_point = if main_header.pruning_point() == HashValue::zero() {
                genesis.block().id()
            } else {
                main_header.pruning_point()
            };

            dag.calc_mergeset_and_tips(
                pruning_point,
                self.config.miner.maximum_parents_count(),
                self.chain_service.get_main().get_genesis_hash(),
            )?
        } else {
            let tips = dag.get_dag_state(genesis.block().id())?.tips;
            let ghostdata = dag.ghostdata(&tips)?;
            MineNewDagBlockInfo {
                tips: tips.clone(),
                blue_blocks: ghostdata.mergeset_blues.as_ref().clone(),
                pruning_point: HashValue::zero(),
            }
        };

        if blue_blocks.is_empty() {
            bail!("failed to get the blue blocks from the DAG");
        }

        let selected_parent = *blue_blocks
            .first()
            .ok_or_else(|| format_err!("the blue blocks must be not be 0!"))?;

        let time_service = self.config.net().time_service();
        let storage = self.storage.clone();
        let vm_metrics = self.vm_metrics.clone();
        let main = BlockChain::new(time_service, selected_parent, storage, vm_metrics, dag)?;

        let epoch = main.epoch().clone();
        let strategy = epoch.strategy();
        let on_chain_block_gas_limit = epoch.block_gas_limit();
        let previous_header = main
            .get_storage()
            .get_block_header_by_hash(selected_parent)?
            .ok_or_else(|| format_err!("BlockHeader should exist by hash: {}", selected_parent))?;
        let next_difficulty = epoch.strategy().calculate_next_difficulty(&main)?;
        let now_milliseconds = main.time_service().now_millis();

        Ok(Box::new(MinerResponse {
            previous_header,
            on_chain_block_gas_limit,
            tips_hash: tips,
            blue_blocks_hash: blue_blocks[1..].to_vec(),
            strategy,
            next_difficulty,
            now_milliseconds,
            pruning_point,
        }))
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
        _ctx: &mut ServiceContext<Self>,
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
            self.chain_service.try_connect(block)?;
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
        if self.chain_service.get_main().status().head().id() == msg.head_hash {
            info!("the branch in chain service is the same as target's branch");
            Ok(())
        } else {
            info!("mock branch in chain service is not the same as target's branch");
            bail!("blockchain in chain service is not the same as target!");
        }
    }
}
