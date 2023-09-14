// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::block_connector::{ExecuteRequest, ResetRequest, WriteBlockChainService};
use crate::sync::{CheckSyncEvent, SyncService};
use crate::tasks::{BlockConnectedEvent, BlockDiskCheckEvent};
use anyhow::{format_err, Result, Ok};
use network_api::PeerProvider;
use starcoin_chain_api::{ConnectBlockError, WriteableChainService, ChainReader};
use starcoin_config::{NodeConfig, G_CRATE_VERSION};
use starcoin_consensus::BlockDAG;
use starcoin_executor::VMMetrics;
use starcoin_logger::prelude::*;
use starcoin_network::NetworkServiceRef;
use starcoin_service_registry::{
    ActorService, EventHandler, ServiceContext, ServiceFactory, ServiceHandler,
};
use starcoin_storage::{BlockStore, Storage};
use starcoin_sync_api::{PeerNewBlock, NewBlockChainRequest};
use starcoin_txpool::TxPoolService;
use starcoin_types::block::ExecutedBlock;
use starcoin_types::sync_status::SyncStatus;
use starcoin_types::system_events::{MinedBlock, SyncStatusChangeEvent, SystemShutdown, NewHeadBlock};
use std::sync::{Arc, Mutex};
use sysinfo::{DiskExt, System, SystemExt};

use super::BlockConnectedRequest;

const DISK_CHECKPOINT_FOR_PANIC: u64 = 1024 * 1024 * 1024 * 3;
const DISK_CHECKPOINT_FOR_WARN: u64 = 1024 * 1024 * 1024 * 5;

pub struct BlockConnectorService {
    chain_service: WriteBlockChainService<TxPoolService>,
    sync_status: Option<SyncStatus>,
    config: Arc<NodeConfig>,
}

impl BlockConnectorService {
    pub fn new(
        chain_service: WriteBlockChainService<TxPoolService>,
        config: Arc<NodeConfig>,
    ) -> Self {
        Self {
            chain_service,
            sync_status: None,
            config,
        }
    }

    pub fn is_synced(&self) -> bool {
        match self.sync_status.as_ref() {
            Some(sync_status) => sync_status.is_synced(),
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

impl ServiceFactory<Self> for BlockConnectorService {
    fn create(ctx: &mut ServiceContext<BlockConnectorService>) -> Result<BlockConnectorService> {
        let config = ctx.get_shared::<Arc<NodeConfig>>()?;
        let bus = ctx.bus_ref().clone();
        let txpool = ctx.get_shared::<TxPoolService>()?;
        let storage = ctx.get_shared::<Arc<Storage>>()?;
        let startup_info = storage
            .get_startup_info()?
            .ok_or_else(|| format_err!("Startup info should exist."))?;
        let vm_metrics = ctx.get_shared_opt::<VMMetrics>()?;
        let dag = ctx.get_shared::<Arc<Mutex<BlockDAG>>>()?;
        let chain_service = WriteBlockChainService::new(
            config.clone(),
            startup_info,
            storage,
            txpool,
            bus,
            vm_metrics,
            dag.clone(),
        )?;

        Ok(Self::new(chain_service, config))
    }
}

impl ActorService for BlockConnectorService {
    fn started(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        //TODO figure out a more suitable value.
        ctx.set_mailbox_capacity(1024);
        ctx.subscribe::<SyncStatusChangeEvent>();
        ctx.subscribe::<MinedBlock>();

        ctx.run_interval(std::time::Duration::from_secs(3), move |ctx| {
            ctx.notify(crate::tasks::BlockDiskCheckEvent {});
        });

        Ok(())
    }

    fn stopped(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.unsubscribe::<SyncStatusChangeEvent>();
        ctx.unsubscribe::<MinedBlock>();
        Ok(())
    }
}

impl EventHandler<Self, BlockDiskCheckEvent> for BlockConnectorService {
    fn handle_event(
        &mut self,
        _: BlockDiskCheckEvent,
        ctx: &mut ServiceContext<BlockConnectorService>,
    ) {
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

impl EventHandler<Self, BlockConnectedEvent> for BlockConnectorService {
    fn handle_event(
        &mut self,
        msg: BlockConnectedEvent,
        _ctx: &mut ServiceContext<BlockConnectorService>,
    ) {
        //because this block has execute at sync task, so just try connect to select head chain.
        //TODO refactor connect and execute

        let block = msg.block;
        if let Err(e) = self.chain_service.try_connect(block, msg.dag_parents) {
            error!("Process connected block error: {:?}", e);
        }
    }
}

impl EventHandler<Self, MinedBlock> for BlockConnectorService {
    fn handle_event(&mut self, msg: MinedBlock, _ctx: &mut ServiceContext<Self>) {
        let MinedBlock(new_block, tips_headers) = msg;
        let id = new_block.header().id();
        debug!("try connect mined block: {}", id);

        match self
            .chain_service
            .try_connect(new_block.as_ref().clone(), tips_headers)
        {
            std::result::Result::Ok(_) => debug!("Process mined block {} success.", id),
            Err(e) => {
                warn!("Process mined block {} fail, error: {:?}", id, e);
            }
        }
    }
}

impl EventHandler<Self, SyncStatusChangeEvent> for BlockConnectorService {
    fn handle_event(&mut self, msg: SyncStatusChangeEvent, _ctx: &mut ServiceContext<Self>) {
        self.sync_status = Some(msg.0);
    }
}

impl EventHandler<Self, PeerNewBlock> for BlockConnectorService {
    fn handle_event(&mut self, msg: PeerNewBlock, ctx: &mut ServiceContext<Self>) {
        if !self.is_synced() {
            debug!("[connector] Ignore PeerNewBlock event because the node has not been synchronized yet.");
            return;
        }
        let peer_id = msg.get_peer_id();
        if let Err(e) = self
            .chain_service
            .try_connect(msg.get_block().clone(), msg.get_dag_parents().clone())
        {
            match e.downcast::<ConnectBlockError>() {
                std::result::Result::Ok(connect_error) => {
                    match connect_error {
                        ConnectBlockError::FutureBlock(block) => {
                            //TODO cache future block
                            if let std::result::Result::Ok(sync_service) = ctx.service_ref::<SyncService>() {
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
        }
    }
}

impl ServiceHandler<Self, ResetRequest> for BlockConnectorService {
    fn handle(
        &mut self,
        msg: ResetRequest,
        _ctx: &mut ServiceContext<BlockConnectorService>,
    ) -> Result<()> {
        self.chain_service
            .reset(msg.block_hash, msg.dag_block_parent)
    }
}

impl ServiceHandler<Self, NewBlockChainRequest> for BlockConnectorService {
    fn handle(
        &mut self,
        msg: NewBlockChainRequest,
        ctx: &mut ServiceContext<BlockConnectorService>,
    ) -> Result<()> {
        let (new_branch, dag_parents, next_tips) = self.chain_service.switch_new_main(msg.new_head_block)?;
        ctx.broadcast(NewHeadBlock(Arc::new(new_branch.head_block()), Some(dag_parents), Some(next_tips)));
        Ok(())
    }
}

impl ServiceHandler<Self, ExecuteRequest> for BlockConnectorService {
    fn handle(
        &mut self,
        msg: ExecuteRequest,
        _ctx: &mut ServiceContext<BlockConnectorService>,
    ) -> Result<ExecutedBlock> {
        self.chain_service
            .execute(msg.block, msg.dag_transaction_parent)
    }
}

impl ServiceHandler<Self, BlockConnectedRequest> for BlockConnectorService {
    fn handle(
        &mut self,
        msg: BlockConnectedRequest,
        _ctx: &mut ServiceContext<BlockConnectorService>,
    ) -> Result<()> {
        //because this block has execute at sync task, so just try connect to select head chain.
        //TODO refactor connect and execute

        let block = msg.block;
        self.chain_service.try_connect(block, msg.dag_parents)
    }
}
