// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use actix::{clock::delay_for, prelude::*};
use anyhow::Result;
use network_rpc_core::server::NetworkRpcServer;
use starcoin_account_service::{AccountEventService, AccountService, AccountStorage};
use starcoin_block_relayer::BlockRelayer;
use starcoin_bus::{Bus, BusActor};
use starcoin_chain::{ChainActor, ChainActorRef};
use starcoin_chain_notify::ChainNotifyHandlerActor;
use starcoin_config::NodeConfig;
use starcoin_dev::playground::PlaygroudService;
use starcoin_genesis::Genesis;
use starcoin_logger::prelude::*;
use starcoin_logger::LoggerHandle;
use starcoin_miner::headblock_pacemaker::HeadBlockPacemaker;
use starcoin_miner::job_bus_client::JobBusClient;
use starcoin_miner::ondemand_pacemaker::OndemandPacemaker;
use starcoin_miner::{CreateBlockTemplateService, MinerClientActor, MinerService};
use starcoin_network::{NetworkAsyncService, PeerMsgBroadcasterActor};
use starcoin_network_rpc_api::gen_client::get_rpc_info;
use starcoin_node_api::message::{NodeRequest, NodeResponse};
use starcoin_rpc_server::module::PubSubService;
use starcoin_rpc_server::RpcActor;
use starcoin_service_registry::bus::BusService;
use starcoin_service_registry::{ActorService, RegistryAsyncService, RegistryService, ServiceRef};
use starcoin_state_service::ChainStateActor;
use starcoin_storage::block_info::BlockInfoStore;
use starcoin_storage::cache_storage::CacheStorage;
use starcoin_storage::db_storage::DBStorage;
use starcoin_storage::storage::StorageInstance;
use starcoin_storage::{BlockStore, Storage};
use starcoin_sync::SyncActor;
use starcoin_sync_api::StartSyncTxnEvent;
use starcoin_txpool::{TxPool, TxPoolService};
use starcoin_types::peer_info::{PeerInfo, RpcInfo};
use starcoin_types::system_events::SystemStarted;
use std::sync::Arc;
use std::time::Duration;

//TODO rework field and order.
pub struct NodeStartedHandle {
    pub config: Arc<NodeConfig>,
    pub bus: Addr<BusActor>,
    pub storage: Arc<Storage>,
    pub chain_arbiter: Arbiter,
    pub chain_actor: ChainActorRef,
    pub sync_actor: Addr<SyncActor<NetworkAsyncService>>,
    pub rpc_actor: Addr<RpcActor>,
    pub miner_client: Option<Addr<MinerClientActor<JobBusClient>>>,
    pub chain_notifier: Addr<ChainNotifyHandlerActor>,
    pub network: NetworkAsyncService,
    pub network_rpc_server: Addr<NetworkRpcServer>,
    pub block_relayer: Addr<BlockRelayer<TxPoolService>>,
    pub peer_msg_broadcaster: Addr<PeerMsgBroadcasterActor>,
    pub txpool: TxPool,
    pub node_addr: Addr<Node>,
}

pub struct Node {
    //TODO remove there fields, after register all service to registry.
    pub chain_arbiter: Arbiter,
    pub chain_actor: ChainActorRef,
    pub sync_actor: Addr<SyncActor<NetworkAsyncService>>,
    pub rpc_actor: Addr<RpcActor>,
    pub miner_client: Option<Addr<MinerClientActor<JobBusClient>>>,
    pub chain_notifier: Addr<ChainNotifyHandlerActor>,
    pub network: NetworkAsyncService,
    pub network_rpc_server: Addr<NetworkRpcServer>,
    pub block_relayer: Addr<BlockRelayer<TxPoolService>>,
    pub txpool: TxPool,
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
                if let Err(e) = self.registry.shutdown_sync() {
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

    let account_service = registry.registry::<AccountService>().await?;
    registry.registry::<AccountEventService>().await?;

    let head_block_hash = *startup_info.get_master();

    let txpool = TxPool::start(
        config.clone(),
        storage.clone(),
        head_block_hash,
        bus.clone(),
    );
    let txpool_service = txpool.get_service();

    // put tx pool service as shared data, remove this after refactor txpool service.
    registry.put_shared(txpool_service.clone()).await?;

    let head_block = match storage.get_block(head_block_hash)? {
        Some(block) => block,
        None => panic!("can't get block by hash {}", head_block_hash),
    };
    let head_block_info = match storage.get_block_info(head_block_hash)? {
        Some(block_info) => block_info,
        None => panic!("can't get block info by hash {}", head_block_hash),
    };
    let peer_id = config
        .network
        .self_peer_id
        .clone()
        .expect("Self peer_id must has been set.");
    let mut rpc_proto_info = Vec::new();
    let chain_rpc_proto_info = get_rpc_info();
    rpc_proto_info.push((
        chain_rpc_proto_info.0.into(),
        RpcInfo::new(chain_rpc_proto_info.1),
    ));
    let self_info = PeerInfo::new_with_proto(
        peer_id.clone(),
        head_block_info.get_total_difficulty(),
        head_block.header().clone(),
        rpc_proto_info,
    );
    let network_config = config.clone();
    let network_bus = bus.clone();

    let (network, rpc_rx) = NetworkAsyncService::start(
        network_config.clone(),
        network_bus.clone(),
        genesis_hash,
        self_info,
    );
    let peer_msg_broadcaster = Arbiter::new()
        .exec({
            let network = network.clone();
            let network_bus = network_bus.clone();
            move || PeerMsgBroadcasterActor::launch(network, network_bus)
        })
        .await?;

    let head_block = storage
        .get_block(*startup_info.get_master())?
        .expect("Head block must exist.");
    let block_relayer = BlockRelayer::new(bus.clone(), txpool.get_service(), network.clone())?;
    let chain_state_service = ChainStateActor::launch(
        bus.clone(),
        storage.clone(),
        Some(head_block.header().state_root()),
    )?;

    let chain_config = config.clone();
    let chain_storage = storage.clone();
    let chain_txpool_service = txpool_service.clone();

    let chain_arbiter = Arbiter::new();
    let chain_startup_info = startup_info.clone();
    let chain_bus = bus.clone();
    let chain = chain_arbiter
        .exec(move || -> Result<ChainActorRef> {
            ChainActor::launch(
                chain_config,
                chain_startup_info,
                chain_storage,
                chain_txpool_service,
                chain_bus,
            )
        })
        .await??;

    // running in background
    let chain_notify_handler = {
        let bus = bus.clone();
        let storage = storage.clone();
        Actor::start_in_arbiter(&Arbiter::new(), |_ctx| {
            ChainNotifyHandlerActor::new(bus, storage)
        })
    };

    // network rpc server
    let network_rpc_server = starcoin_network_rpc::start_network_rpc_server(
        rpc_rx,
        chain.clone(),
        storage.clone(),
        chain_state_service.clone(),
        txpool_service.clone(),
    )?;

    info!("Self peer_id is: {}", peer_id.to_base58());
    info!(
        "Self address is: {}",
        config
            .network
            .self_address
            .as_ref()
            .expect("Self connect address must has been set.")
    );
    let peer_id = Arc::new(peer_id);
    let sync_config = config.clone();
    let sync_bus = bus.clone();
    let sync_chain = chain.clone();
    let sync_txpool = txpool_service.clone();
    let sync_network = network.clone();
    let sync_storage = storage.clone();
    let sync_startup_info = startup_info.clone();
    let sync = Arbiter::new()
        .exec(move || -> Result<Addr<SyncActor<NetworkAsyncService>>> {
            SyncActor::launch(
                sync_config,
                sync_bus,
                peer_id,
                sync_chain,
                sync_txpool,
                sync_network,
                sync_storage,
                sync_startup_info,
            )
        })
        .await??;

    delay_for(Duration::from_secs(1)).await;

    registry.registry::<CreateBlockTemplateService>().await?;
    registry.registry::<MinerService>().await?;

    let miner_client_config = config.miner.client_config.clone();
    let consensus_strategy = config.net().consensus();
    let job_client = JobBusClient::new(bus.clone());
    let miner_client = if config.miner.enable_miner_client {
        Some(
            Arbiter::new()
                .exec(move || {
                    MinerClientActor::new(miner_client_config, consensus_strategy, job_client)
                        .start()
                })
                .await?,
        )
    } else {
        None
    };

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

    registry.registry::<OndemandPacemaker>().await?;
    registry.registry::<HeadBlockPacemaker>().await?;

    let node = Node {
        chain_arbiter: chain_arbiter.clone(),
        chain_actor: chain.clone(),
        sync_actor: sync.clone(),
        rpc_actor: json_rpc.clone(),
        miner_client: miner_client.clone(),
        chain_notifier: chain_notify_handler.clone(),
        network: network.clone(),
        network_rpc_server: network_rpc_server.clone(),
        block_relayer: block_relayer.clone(),
        txpool: txpool.clone(),
        registry,
    };
    let node_addr = node.start();
    //TODO remove NodeStartedHandle after refactor finished.
    Ok(NodeStartedHandle {
        config,
        bus,
        storage,
        chain_arbiter,
        chain_actor: chain,
        sync_actor: sync,
        rpc_actor: json_rpc,
        miner_client,
        chain_notifier: chain_notify_handler,
        network,
        network_rpc_server,
        block_relayer,
        peer_msg_broadcaster,
        txpool,
        node_addr,
    })
}
