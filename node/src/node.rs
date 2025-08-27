// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::metrics::{MetricsPushActorService, MetricsServerActorService};
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
use starcoin_logger::structured_log::init_slog_logger;
use starcoin_logger::LoggerHandle;
use starcoin_metrics::metrics::VMMetrics;
use starcoin_miner::generate_block_event_pacemaker::GenerateBlockEventPacemaker;
use starcoin_miner::{BlockBuilderService, MinerService};
use starcoin_miner_client::job_bus_client::JobBusClient;
use starcoin_miner_client::miner::MinerClientService;
use starcoin_network::NetworkActorService;
use starcoin_network_rpc::NetworkRpcService;
use starcoin_node_api::errors::NodeStartError;
use starcoin_node_api::message::{NodeRequest, NodeResponse};
use starcoin_rpc_server::module::{PubSubService, PubSubServiceFactory};
use starcoin_rpc_server::service::RpcService;
use starcoin_service_registry::bus::{Bus, BusService};
use starcoin_service_registry::{
    ActorService, EventHandler, RegistryAsyncService, RegistryService, ServiceContext,
    ServiceFactory, ServiceHandler, ServiceRef,
};
use starcoin_state_service::ChainStateService;
use starcoin_storage::storage::DEFAULT_UPGRADE_BATCH_SIZE;
use starcoin_storage::{
    block_info::BlockInfoStore, cache_storage::CacheStorage, db_storage::DBStorage,
    errors::StorageInitError, metrics::StorageMetrics, storage::StorageInstance, BlockStore,
    Storage,
};
use starcoin_stratum::service::{StratumService, StratumServiceFactory};
use starcoin_stratum::stratum::{Stratum, StratumFactory};
use starcoin_sync::announcement::AnnouncementService;
use starcoin_sync::block_connector::{BlockConnectorService, ExecuteService, ResetRequest};
use starcoin_sync::sync::SyncService;
use starcoin_sync::txn_sync::TxnSyncService;
use starcoin_txpool::{TxPoolActorService, TxPoolService};
use starcoin_types::system_events::{SystemShutdown, SystemStarted};
use starcoin_vm2_account_service::{
    AccountEventService as AccountEventService2, AccountService as AccountService2,
    AccountStorage as AccountStorage2,
};
use starcoin_vm2_state_service::ChainStateService as ChainStateService2;
use starcoin_vm2_storage::{
    cache_storage::CacheStorage as CacheStorage2, db_storage::DBStorage as DBStorage2,
    storage::StorageInstance as StorageInstance2, Storage as Storage2,
};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

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

impl ActorService for NodeService {
    fn started(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.subscribe::<SystemShutdown>();
        Ok(())
    }

    fn stopped(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.unsubscribe::<SystemShutdown>();
        Ok(())
    }
}

impl EventHandler<Self, SystemShutdown> for NodeService {
    fn handle_event(&mut self, _: SystemShutdown, _: &mut ServiceContext<Self>) {
        self.shutdown_system();
    }
}

impl ServiceHandler<Self, NodeRequest> for NodeService {
    fn handle(
        &mut self,
        msg: NodeRequest,
        ctx: &mut ServiceContext<NodeService>,
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
                self.shutdown_system();
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
            NodeRequest::ResetNode(block_hash) => {
                let connect_service = ctx
                    .service_ref::<BlockConnectorService<TxPoolService>>()?
                    .clone();
                let fut = async move {
                    info!("Prepare to reset node startup info to {}", block_hash);
                    connect_service.send(ResetRequest { block_hash }).await?
                };
                let receiver = ctx.exec(fut);
                NodeResponse::AsyncResult(receiver)
            }
            NodeRequest::ReExecuteBlock(block_hash) => {
                let sync_service = ctx.service_ref::<SyncService>()?.clone();
                let fut = async move {
                    info!("Prepare to re execute block {}", block_hash);
                    sync_service.notify(starcoin_sync_api::SyncSpecificTargretRequest {
                        block: None,
                        block_id: block_hash,
                        peer_id: None,
                    })?;
                    Ok(())
                };
                let receiver = ctx.exec(fut);
                NodeResponse::AsyncResult(receiver)
            }
            NodeRequest::DeleteBlock(block_hash) => {
                let storage = self
                    .registry
                    .get_shared_sync::<Arc<Storage>>()
                    .expect("Storage must exist.");
                info!("Prepare to delete block {}", block_hash);
                NodeResponse::Result(
                    storage
                        .delete_block_info(block_hash)
                        .and_then(|_| storage.delete_block(block_hash)),
                )
            }
            NodeRequest::DeleteFailedBlock(block_hash) => {
                let storage = self
                    .registry
                    .get_shared_sync::<Arc<Storage>>()
                    .expect("Storage must exist.");
                info!("Prepare to delete failed block {:?}", block_hash);
                NodeResponse::Result(storage.delete_failed_block(block_hash))
            }
        })
    }
}

impl NodeService {
    pub fn launch(
        config: Arc<NodeConfig>,
        logger_handle: Arc<LoggerHandle>,
    ) -> Result<NodeHandle, NodeStartError> {
        info!("Final data-dir is : {:?}", config.data_dir());
        if let Some(log_path) = config.logger.get_log_path() {
            info!("Write log to file: {:?}", log_path);
            logger_handle.enable_file(
                log_path.clone(),
                config.logger.max_file_size(),
                config.logger.max_backup(),
            );
            //config slog
            if let Err(e) = init_slog_logger(log_path, !config.logger.disable_stderr()) {
                warn!("slog config error: {}", e);
            }
        }

        if config.logger.disable_stderr() {
            logger_handle.disable_stderr();
        } else {
            logger_handle.enable_stderr();
        }

        // StarcoinVM::set_concurrency_level_once(num_cpus::get());
        let (start_sender, start_receiver) = oneshot::channel();
        let join_handle = timeout_join_handler::spawn(move || {
            let system = System::with_tokio_rt(|| {
                tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .on_thread_stop(|| debug!("main thread stopped"))
                    .thread_name("main")
                    .build()
                    .expect("failed to create tokio runtime for main")
            });
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
        let (registry, node_service) = block_on(start_receiver).expect("Wait node start error.")?;
        Ok(NodeHandle::new(join_handle, node_service, registry))
    }

    async fn init_system(
        config: Arc<NodeConfig>,
        logger_handle: Arc<LoggerHandle>,
    ) -> Result<(ServiceRef<RegistryService>, ServiceRef<NodeService>)> {
        let registry = RegistryService::launch();

        registry.put_shared(config.clone()).await?;
        registry.put_shared(logger_handle).await?;
        let vm_metrics = config
            .metrics
            .registry()
            .and_then(|registry| VMMetrics::register(registry).ok());
        if let Some(vm_metrics) = vm_metrics {
            registry.put_shared(vm_metrics).await?;
        }
        let bus = registry.service_ref::<BusService>().await?;
        let storage_metrics = config
            .metrics
            .registry()
            .and_then(|registry| StorageMetrics::register(registry).ok());
        info!(
            "rocksdb max open files {}",
            config.storage.rocksdb_config().max_open_files
        );
        let mut storage_instance = StorageInstance::new_cache_and_db_instance(
            CacheStorage::new_with_capacity(config.storage.cache_size(), storage_metrics.clone()),
            DBStorage::new(
                config.storage.dir(),
                config.storage.rocksdb_config(),
                storage_metrics,
            )?,
        );

        let config2 = starcoin_vm2_storage::db_storage::RocksdbConfig::new(
            config.storage.rocksdb_config().max_open_files,
            config.storage.rocksdb_config().max_total_wal_size,
            config.storage.rocksdb_config().bytes_per_sync,
            config.storage.rocksdb_config().wal_bytes_per_sync,
        );
        let storage_instance2 = StorageInstance2::new_cache_and_db_instance(
            CacheStorage2::new_with_capacity(config.storage.cache_size(), None),
            DBStorage2::new(config.storage.dir(), config2.clone(), None)?,
        );

        let start_time = SystemTime::now();
        storage_instance.check_upgrade(DEFAULT_UPGRADE_BATCH_SIZE)?;

        let upgrade_time = SystemTime::now().duration_since(start_time)?;
        let storage = Arc::new(Storage::new(storage_instance)?);
        let storage2 = Arc::new(Storage2::new(storage_instance2)?);
        registry.put_shared(storage.clone()).await?;
        registry.put_shared(storage2.clone()).await?;
        // Also share storage2 as Arc<dyn Store2> for PruningPointService
        registry
            .put_shared(storage2.clone() as Arc<dyn starcoin_vm2_storage::Store>)
            .await?;

        // Initialize DAG
        let dag_storage = starcoin_dag::consensusdb::prelude::FlexiDagStorage::create_from_path(
            config.storage.dag_dir(),
            config.storage.clone().into(),
        )?;
        // Get K from genesis config instead of using constant directly
        let k = config
            .net()
            .genesis_config2()
            .consensus_config
            .base_max_uncles_per_block;
        let dag = starcoin_dag::blockdag::BlockDAG::new(
            starcoin_types::blockhash::KType::try_from(k)?,
            config.miner.dag_merge_depth(),
            config.miner.maximum_parents_count(),
            dag_storage.clone(),
        );
        registry.put_shared(dag.clone()).await?;

        let (chain_info, genesis) = Genesis::init_and_check_storage(
            config.net(),
            storage.clone(),
            storage2.clone(),
            dag.clone(),
            config.data_dir(),
        )?;

        info!(
            "Start node with chain info: {}, number {}, dragon fork disabled, upgrade_time cost {} secs, ",
            chain_info,
            chain_info.status().head().number(),
            upgrade_time.as_secs()
        );

        registry.put_shared(genesis).await?;

        // Register PruningPointService after genesis is available
        use starcoin_dag::service::pruning_point_service::PruningPointService;
        registry.register::<PruningPointService>().await?;

        let node_service = registry.register::<NodeService>().await?;

        registry.register::<ChainStateService>().await?;
        registry.register::<ChainStateService2>().await?;

        let vault_config = &config.vault;
        let account_storage =
            AccountStorage::create_from_path(vault_config.dir(), config.storage.rocksdb_config())?;
        registry
            .put_shared::<AccountStorage>(account_storage.clone())
            .await?;

        let account_storage2 = AccountStorage2::create_from_path(vault_config.dir2(), config2)?;
        registry
            .put_shared::<AccountStorage2>(account_storage2.clone())
            .await?;

        registry.register::<AccountService>().await?;
        registry.register::<AccountEventService>().await?;

        registry.register::<AccountService2>().await?;
        registry.register::<AccountEventService2>().await?;

        let txpool_service = registry.register::<TxPoolActorService>().await?;

        //wait TxPoolService put shared..
        // todo: find a way to make sure TxPoolService is ready.
        Delay::new(Duration::from_secs(1)).await;
        // TxPoolActorService auto put shared TxPoolService,
        registry.register::<ChainReaderService>().await?;

        registry.register::<ChainNotifyHandlerService>().await?;

        registry.register::<ExecuteService>().await?;
        registry
            .register::<BlockConnectorService<TxPoolService>>()
            .await?;
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

        // Create NewHeaderChannel for miner services
        use starcoin_miner::{NewHeaderChannel, NewHeaderService};
        registry.put_shared(NewHeaderChannel::new()).await?;
        registry.register::<NewHeaderService>().await?;

        registry.register::<BlockBuilderService>().await?;
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

        // start metrics server
        if !config.metrics.disable_metrics() {
            registry.register::<MetricsServerActorService>().await?;
        }
        // start metrics push service
        if config.metrics.push_config.is_config() {
            registry.register::<MetricsPushActorService>().await?;
        }

        Ok((registry, node_service))
    }

    fn shutdown_system(&self) {
        if let Err(e) = self.registry.shutdown_system_sync() {
            error!("Shutdown registry error: {}", e);
        };
        //wait a seconds for registry shutdown, then stop System.
        std::thread::sleep(Duration::from_millis(2000));
        System::current().stop();
    }
}
