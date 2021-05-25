// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::metrics::MetricsActorService;
use crate::network_service_factory::NetworkServiceFactory;
use crate::peer_message_handler::NodePeerMessageHandler;
use crate::rpc_service_factory::RpcServiceFactory;
use crate::NodeHandle;
use actix::prelude::*;
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
use starcoin_logger::structured_log::set_global_logger;
use starcoin_logger::LoggerHandle;
use starcoin_miner::generate_block_event_pacemaker::GenerateBlockEventPacemaker;
use starcoin_miner::job_bus_client::JobBusClient;
use starcoin_miner::{CreateBlockTemplateService, MinerClientService, MinerService};
use starcoin_network::NetworkActorService;
use starcoin_network_rpc::NetworkRpcService;
use starcoin_node_api::errors::NodeStartError;
use starcoin_node_api::message::{NodeRequest, NodeResponse};
use starcoin_rpc_server::module::{PubSubService, PubSubServiceFactory};
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
use starcoin_stratum::service::{StratumService, StratumServiceFactory};
use starcoin_stratum::stratum::{Stratum, StratumFactory};
use starcoin_sync::announcement::AnnouncementService;
use starcoin_sync::block_connector::BlockConnectorService;
use starcoin_sync::sync::SyncService;
use starcoin_sync::txn_sync::TxnSyncService;
use starcoin_txpool::TxPoolActorService;
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
    fn handle(
        &mut self,
        msg: NodeRequest,
        _ctx: &mut ServiceContext<NodeService>,
    ) -> Result<NodeResponse> {
        Ok(match msg {
            NodeRequest::ListService => NodeResponse::Services(self.registry.list_service_sync()?),
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
            NodeRequest::CheckService(service_name) => {
                info!(
                    "Receive StartService request, try to start service {:?}",
                    service_name
                );
                NodeResponse::ServiceStatus(
                    self.registry
                        .check_service_status_sync(service_name.as_str())?,
                )
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
                    .stop_service_sync(GenerateBlockEventPacemaker::service_name()),
            ),
            NodeRequest::StartPacemaker => NodeResponse::Result(
                self.registry
                    .start_service_sync(GenerateBlockEventPacemaker::service_name()),
            ),
        })
    }
}

impl NodeService {
    pub fn launch(
        config: Arc<NodeConfig>,
        logger_handle: Arc<LoggerHandle>,
    ) -> Result<NodeHandle, NodeStartError> {
        info!("Final data-dir is : {:?}", config.data_dir());
        if let Some((log_path, slog_path)) = config.logger.get_log_path() {
            info!("Write log to file: {:?}", log_path);
            logger_handle.enable_file(
                log_path,
                slog_path.clone(),
                config.logger.max_file_size(),
                config.logger.max_backup(),
            );
            //config slog
            info!("Write slog to file: {:?}", slog_path);
            if set_global_logger(
                config.logger.get_slog_is_sync(),
                Some(config.logger.get_slog_chan_size()),
                slog_path,
            )
            .is_ok()
            {
                info!("slog config success.");
            } else {
                warn!("slog config error.");
            }
        }

        if config.logger.disable_stderr() {
            logger_handle.disable_stderr();
        } else {
            logger_handle.enable_stderr();
        }

        // start metric server
        if let Some(metrics_address) = config.metrics.metrics_address() {
            starcoin_metrics::metric_server::start_server(metrics_address);
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

        registry.put_shared(config.clone()).await?;
        registry.put_shared(logger_handle).await?;

        let bus = registry.service_ref::<BusService>().await?;
        let storage = Arc::new(Storage::new(StorageInstance::new_cache_and_db_instance(
            CacheStorage::new_with_capacity(config.storage.cache_size()),
            DBStorage::new(config.storage.dir(), config.storage.rocksdb_config())?,
        ))?);
        registry.put_shared(storage.clone()).await?;
        let (chain_info, genesis) =
            Genesis::init_and_check_storage(config.net(), storage.clone(), config.data_dir())?;

        info!("Start node with chain info: {}", chain_info);

        registry.put_shared(genesis).await?;

        let node_service = registry.register::<NodeService>().await?;

        registry.register::<ChainStateService>().await?;

        let vault_config = &config.vault;
        let account_storage =
            AccountStorage::create_from_path(vault_config.dir(), config.storage.rocksdb_config())?;
        registry
            .put_shared::<AccountStorage>(account_storage.clone())
            .await?;

        registry.register::<AccountService>().await?;
        registry.register::<AccountEventService>().await?;

        let txpool_service = registry.register::<TxPoolActorService>().await?;

        //wait TxPoolService put shared..
        Delay::new(Duration::from_millis(200)).await;
        // TxPoolActorService auto put shared TxPoolService,

        registry.register::<ChainReaderService>().await?;

        registry.register::<ChainNotifyHandlerService>().await?;

        registry.register::<BlockConnectorService>().await?;
        registry.register::<SyncService>().await?;

        let block_relayer = registry.register::<BlockRelayer>().await?;

        registry.register::<NetworkRpcService>().await?;
        let announcement_service = registry.register::<AnnouncementService>().await?;

        NodePeerMessageHandler::new(txpool_service, block_relayer, announcement_service);

        registry
            .register_by_factory::<NetworkActorService, NetworkServiceFactory>()
            .await?;
        //wait Network service init
        Delay::new(Duration::from_millis(200)).await;

        registry.register::<TxnSyncService>().await?;

        let peer_id = config.network.self_peer_id();

        info!("Self peer_id is: {}", peer_id.to_base58());
        info!("Self address is: {}", config.network.self_address());

        registry.register::<CreateBlockTemplateService>().await?;
        let miner_service = registry.register::<MinerService>().await?;

        if let Some(miner_client_config) = config.miner.miner_client_config() {
            registry.put_shared(miner_client_config).await?;
            let job_client =
                JobBusClient::new(miner_service, bus.clone(), config.net().time_service());
            registry.put_shared(job_client).await?;
            registry
                .register::<MinerClientService<JobBusClient>>()
                .await?;
        } else {
            info!("Config.miner.enable_miner_client is false, No in process MinerClient.");
        }

        registry
            .register_by_factory::<Stratum, StratumFactory>()
            .await?;

        registry.register::<GenerateBlockEventPacemaker>().await?;

        // start metrics push service
        if config.metrics.push_config.is_config() {
            registry.register::<MetricsActorService>().await?;
        }
        // wait for service init.
        Delay::new(Duration::from_millis(1000)).await;

        bus.broadcast(SystemStarted)?;

        registry
            .register_by_factory::<PubSubService, PubSubServiceFactory>()
            .await?;
        registry
            .register_by_factory::<RpcService, RpcServiceFactory>()
            .await?;
        registry
            .register_by_factory::<StratumService, StratumServiceFactory>()
            .await?;

        Ok((registry, node_service))
    }
}
