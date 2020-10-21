// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::rpc_service_factory::RpcServiceFactory;
use crate::NodeHandle;
use actix::{clock::delay_for, prelude::*};
use anyhow::Result;
use futures::channel::oneshot;
use futures::executor::block_on;
use futures_timer::Delay;
use starcoin_account_service::{AccountEventService, AccountService, AccountStorage};
use starcoin_block_relayer::BlockRelayer;
use starcoin_chain_notify::ChainNotifyHandlerService;
use starcoin_chain_service::ChainReaderService;
use starcoin_config::NodeConfig;
use starcoin_genesis::{Genesis, GenesisError};
use starcoin_logger::prelude::*;
use starcoin_logger::LoggerHandle;
use starcoin_miner::headblock_pacemaker::HeadBlockPacemaker;
use starcoin_miner::job_bus_client::JobBusClient;
use starcoin_miner::ondemand_pacemaker::OndemandPacemaker;
use starcoin_miner::{CreateBlockTemplateService, MinerClientService, MinerService};
use starcoin_network::{NetworkAsyncService, PeerMsgBroadcasterService};
use starcoin_network_rpc::NetworkRpcService;
use starcoin_node_api::errors::NodeStartError;
use starcoin_node_api::message::{NodeRequest, NodeResponse};
use starcoin_rpc_server::service::RpcService;
use starcoin_service_registry::bus::{Bus, BusService};
use starcoin_service_registry::{
    ActorService, RegistryAsyncService, RegistryService, ServiceContext, ServiceFactory,
    ServiceHandler, ServiceRef,
};
use starcoin_state_service::ChainStateService;
use starcoin_storage::cache_storage::CacheStorage;
use starcoin_storage::db_storage::DBStorage;
use starcoin_storage::errors::StorageInitError;
use starcoin_storage::storage::StorageInstance;
use starcoin_storage::Storage;
use starcoin_sync::download::DownloadService;
use starcoin_sync::txn_sync::TxnSyncService;
use starcoin_sync::SyncService;
use starcoin_sync_api::StartSyncTxnEvent;
use starcoin_txpool::{TxPoolActorService, TxPoolService};
use starcoin_types::system_events::SystemStarted;
use std::sync::Arc;
use std::time::Duration;

pub struct NodeService {
    registry: ServiceRef<RegistryService>,
}

impl ServiceFactory<Self> for NodeService {
    fn create(ctx: &mut ServiceContext<NodeService>) -> Result<NodeService> {
        Ok(Self {
            registry: ctx.registry_ref().clone(),
        })
    }
}

impl ActorService for NodeService {}

impl ServiceHandler<Self, NodeRequest> for NodeService {
    fn handle(&mut self, msg: NodeRequest, _ctx: &mut ServiceContext<NodeService>) -> NodeResponse {
        match msg {
            NodeRequest::ListService => {
                NodeResponse::Services(self.registry.list_service_sync().unwrap_or_default())
            }
            NodeRequest::StopService(service_name) => {
                info!(
                    "Receive StopService request, try to stop service {:?}",
                    service_name
                );
                NodeResponse::Result(self.registry.stop_service_sync(service_name.as_str()))
            }
            NodeRequest::StartService(service_name) => {
                info!(
                    "Receive StartService request, try to start service {:?}",
                    service_name
                );
                NodeResponse::Result(self.registry.start_service_sync(service_name.as_str()))
            }
            NodeRequest::ShutdownSystem => {
                info!("Receive StopSystem request, try to stop system.");
                if let Err(e) = self.registry.shutdown_system_sync() {
                    error!("Shutdown registry error: {}", e);
                };
                //wait a seconds for registry shutdown, then stop System.
                std::thread::sleep(Duration::from_millis(2000));
                System::current().stop();
                NodeResponse::Result(Ok(()))
            }
            NodeRequest::StopPacemaker => NodeResponse::Result(
                self.registry
                    .stop_service_sync(HeadBlockPacemaker::service_name())
                    .and_then(|_| {
                        self.registry
                            .stop_service_sync(OndemandPacemaker::service_name())
                    }),
            ),
            NodeRequest::StartPacemaker => NodeResponse::Result(
                self.registry
                    .start_service_sync(HeadBlockPacemaker::service_name())
                    .and_then(|_| {
                        self.registry
                            .start_service_sync(OndemandPacemaker::service_name())
                    }),
            ),
        }
    }
}

impl NodeService {
    pub fn launch(
        config: Arc<NodeConfig>,
        logger_handle: Arc<LoggerHandle>,
    ) -> Result<NodeHandle, NodeStartError> {
        info!("Final data-dir is : {:?}", config.data_dir());
        if config.logger.enable_file() {
            let file_log_path = config.logger.get_log_path();
            info!("Write log to file: {:?}", file_log_path);
            logger_handle.enable_file(
                file_log_path,
                config.logger.max_file_size,
                config.logger.max_backup,
            );
        }
        if config.logger.enable_stderr {
            logger_handle.enable_stderr();
        } else {
            logger_handle.disable_stderr();
        }

        // start metric server
        if config.metrics.enable_metrics {
            starcoin_metrics::metric_server::start_server(
                config.metrics.address.clone(),
                config.metrics.port,
            );
        }

        let (start_sender, start_receiver) = oneshot::channel();
        let join_handle = timeout_join_handler::spawn(move || {
            let mut system = System::builder().stop_on_panic(true).name("main").build();
            system.block_on(async {
                match Self::init_system(config, logger_handle).await {
                    Err(e) => {
                        let node_start_err = match e.downcast::<GenesisError>() {
                            Ok(e) => NodeStartError::GenesisError(e),
                            Err(e) => match e.downcast::<StorageInitError>() {
                                Ok(e) => NodeStartError::StorageInitError(e),
                                Err(e) => NodeStartError::Other(e),
                            },
                        };
                        if start_sender.send(Err(node_start_err)).is_err() {
                            info!("Start send error.");
                        };
                    }
                    Ok(registry) => {
                        if start_sender.send(Ok(registry)).is_err() {
                            info!("Start send error.");
                        }
                    }
                };
            });
            system.run().map_err(|e| e.into())
        });
        let (registry, node_service) =
            block_on(async { start_receiver.await }).expect("Wait node start error.")?;
        Ok(NodeHandle::new(join_handle, node_service, registry))
    }

    async fn init_system(
        config: Arc<NodeConfig>,
        logger_handle: Arc<LoggerHandle>,
    ) -> Result<(ServiceRef<RegistryService>, ServiceRef<NodeService>)> {
        let registry = RegistryService::launch();

        let node_service = registry.register::<NodeService>().await?;

        registry.put_shared(config.clone()).await?;
        registry.put_shared(logger_handle).await?;

        let bus = registry.service_ref::<BusService>().await?;
        let storage = Arc::new(Storage::new(StorageInstance::new_cache_and_db_instance(
            CacheStorage::new(),
            DBStorage::new(config.storage.dir())?,
        ))?);
        registry.put_shared(storage.clone()).await?;
        let (startup_info, genesis) =
            Genesis::init_and_check_storage(config.net(), storage.clone(), config.data_dir())?;

        info!("Start node with startup info: {}", startup_info);
        let genesis_hash = genesis.block().header().id();
        registry.put_shared(genesis).await?;

        registry.register::<ChainStateService>().await?;

        let vault_config = &config.vault;
        let account_storage = AccountStorage::create_from_path(vault_config.dir())?;
        registry
            .put_shared::<AccountStorage>(account_storage.clone())
            .await?;

        registry.register::<AccountService>().await?;
        registry.register::<AccountEventService>().await?;

        registry.register::<TxPoolActorService>().await?;

        //wait TxPoolService put shared..
        Delay::new(Duration::from_millis(200)).await;
        // TxPoolActorService auto put shared TxPoolService,
        registry.get_shared::<TxPoolService>().await?;

        registry.register::<ChainReaderService>().await?;

        registry.register::<ChainNotifyHandlerService>().await?;

        let network_rpc_service = registry.register::<NetworkRpcService>().await?;

        let network = NetworkAsyncService::start(
            config.clone(),
            genesis_hash,
            bus.clone(),
            storage.clone(),
            network_rpc_service,
        )?;
        registry.put_shared(network.clone()).await?;

        registry.register::<PeerMsgBroadcasterService>().await?;
        registry.register::<BlockRelayer>().await?;

        let peer_id = config.network.self_peer_id()?;

        info!("Self peer_id is: {}", peer_id.to_base58());
        info!(
            "Self address is: {}",
            config
                .network
                .self_address
                .as_ref()
                .expect("Self connect address must has been set.")
        );

        registry.register::<TxnSyncService>().await?;
        registry.register::<DownloadService>().await?;
        registry.register::<SyncService>().await?;

        delay_for(Duration::from_secs(1)).await;

        registry.register::<CreateBlockTemplateService>().await?;
        registry.register::<MinerService>().await?;

        if config.miner.enable_miner_client {
            let miner_client_config = config.miner.client_config.clone();
            registry.put_shared(miner_client_config).await?;
            let job_client = JobBusClient::new(bus.clone(), config.net().time_service());
            registry.put_shared(job_client).await?;
            registry
                .register::<MinerClientService<JobBusClient>>()
                .await?;
        } else {
            info!("Config.miner.enable_miner_client is false, No in process MinerClient.");
        }

        bus.broadcast(StartSyncTxnEvent)?;
        bus.broadcast(SystemStarted)?;

        registry.register::<OndemandPacemaker>().await?;
        registry.register::<HeadBlockPacemaker>().await?;

        registry
            .register_by_factory::<RpcService, RpcServiceFactory>()
            .await?;

        Ok((registry, node_service))
    }
}
