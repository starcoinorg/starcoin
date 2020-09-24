// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use actix::{clock::delay_for, prelude::*};
use anyhow::Result;
use futures_timer::Delay;
use starcoin_account_service::{AccountEventService, AccountService, AccountStorage};
use starcoin_block_relayer::BlockRelayer;
use starcoin_bus::{Bus, BusActor};
use starcoin_chain_notify::ChainNotifyHandlerService;
use starcoin_chain_service::ChainReaderService;
use starcoin_config::NodeConfig;
use starcoin_dev::playground::PlaygroudService;
use starcoin_genesis::Genesis;
use starcoin_logger::prelude::*;
use starcoin_logger::LoggerHandle;
use starcoin_miner::headblock_pacemaker::HeadBlockPacemaker;
use starcoin_miner::job_bus_client::JobBusClient;
use starcoin_miner::ondemand_pacemaker::OndemandPacemaker;
use starcoin_miner::{CreateBlockTemplateService, MinerClientService, MinerService};
use starcoin_network::{NetworkAsyncService, PeerMsgBroadcasterService};
use starcoin_network_rpc::NetworkRpcService;
use starcoin_node_api::message::{NodeRequest, NodeResponse};
use starcoin_rpc_server::module::PubSubService;
use starcoin_rpc_server::RpcActor;
use starcoin_service_registry::bus::BusService;
use starcoin_service_registry::{ActorService, RegistryAsyncService, RegistryService, ServiceRef};
use starcoin_state_service::ChainStateService;
use starcoin_storage::cache_storage::CacheStorage;
use starcoin_storage::db_storage::DBStorage;
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

//TODO rework field and order.
pub struct NodeStartedHandle {
    pub config: Arc<NodeConfig>,
    pub bus: Addr<BusActor>,
    pub storage: Arc<Storage>,
    pub rpc_actor: Addr<RpcActor>,
    pub network: NetworkAsyncService,
    pub node_addr: Addr<Node>,
    pub registry: ServiceRef<RegistryService>,
}

impl NodeStartedHandle {
    pub async fn chain_service(&self) -> ServiceRef<ChainReaderService> {
        self.registry
            .service_ref::<ChainReaderService>()
            .await
            .expect("Get ChainReaderService should success.")
    }
}

pub struct Node {
    pub rpc_actor: Addr<RpcActor>,
    pub network: NetworkAsyncService,
    pub registry: ServiceRef<RegistryService>,
}

impl Actor for Node {
    type Context = Context<Self>;
}

impl Handler<NodeRequest> for Node {
    type Result = MessageResult<NodeRequest>;

    fn handle(&mut self, msg: NodeRequest, _ctx: &mut Self::Context) -> Self::Result {
        MessageResult(match msg {
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
            NodeRequest::StopSystem => {
                info!("Receive StopSystem request, try to stop system.");
                if let Err(e) = self.registry.shutdown_system_sync() {
                    error!("Shutdown registry error: {}", e);
                };
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
        })
    }
}

pub async fn start(
    config: Arc<NodeConfig>,
    logger_handle: Option<Arc<LoggerHandle>>,
) -> Result<NodeStartedHandle> {
    let registry = RegistryService::launch();
    registry.put_shared(config.clone()).await?;
    let new_bus = registry.service_ref::<BusService>().await?;
    let bus = BusActor::launch2(new_bus);
    registry.put_shared(bus.clone()).await?;
    let storage = Arc::new(Storage::new(StorageInstance::new_cache_and_db_instance(
        CacheStorage::new(),
        DBStorage::new(config.storage.dir()),
    ))?);
    registry.put_shared(storage.clone()).await?;
    let (startup_info, genesis_hash) =
        Genesis::init_and_check_storage(config.net(), storage.clone(), config.data_dir())?;

    info!("Start node with startup info: {}", startup_info);

    let vault_config = &config.vault;
    let account_storage = AccountStorage::create_from_path(vault_config.dir())?;
    registry
        .put_shared::<AccountStorage>(account_storage.clone())
        .await?;

    let account_service = registry.register::<AccountService>().await?;
    registry.register::<AccountEventService>().await?;

    registry.register::<TxPoolActorService>().await?;

    //wait TxPoolService put shared..
    Delay::new(Duration::from_millis(200)).await;
    // TxPoolActorService auto put shared TxPoolService,
    let txpool_service = registry.get_shared::<TxPoolService>().await?;

    let chain_state_service = registry.register::<ChainStateService>().await?;

    let chain = registry.register::<ChainReaderService>().await?;
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

    let miner_client_config = config.miner.client_config.clone();
    registry.put_shared(miner_client_config).await?;
    let job_client = JobBusClient::new(bus.clone(), config.net().consensus());
    registry.put_shared(job_client).await?;
    registry
        .register::<MinerClientService<JobBusClient>>()
        .await?;
    if !config.miner.enable_miner_client {
        info!("Config.miner.enable_miner_client is false, so stop MinerClientService.");
        registry
            .stop_service(MinerClientService::<JobBusClient>::service_name())
            .await?;
    }

    let (json_rpc, _io_handler) = RpcActor::launch(
        config.clone(),
        bus.clone(),
        txpool_service.clone(),
        chain.clone(),
        account_service,
        chain_state_service,
        Some(PlaygroudService::new(storage.clone())),
        Some(PubSubService::new(bus.clone(), txpool_service)),
        Some(network.clone()),
        logger_handle,
    )?;
    bus.clone().broadcast(StartSyncTxnEvent).await.unwrap();
    bus.clone().broadcast(SystemStarted).await?;

    registry.register::<OndemandPacemaker>().await?;
    registry.register::<HeadBlockPacemaker>().await?;

    let node = Node {
        rpc_actor: json_rpc.clone(),
        network: network.clone(),
        registry: registry.clone(),
    };
    let node_addr = node.start();
    //TODO remove NodeStartedHandle after refactor finished.
    Ok(NodeStartedHandle {
        config,
        bus,
        storage,
        rpc_actor: json_rpc,
        network,
        node_addr,
        registry,
    })
}
