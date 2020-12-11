// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::service::NetworkActorService;
use crate::worker::RPC_PROTOCOL_PREFIX;
use crate::PeerMessage;
use anyhow::{format_err, Result};
use futures::future::BoxFuture;
use futures::{FutureExt, TryFutureExt};
use network_api::messages::NotificationMessage;
use network_api::{messages::TransactionsMessage, NetworkService, PeerProvider, ReputationChange};
use network_p2p_types::network_state::NetworkState;
use network_p2p_types::Multiaddr;
use network_rpc_core::RawRpcClient;
use starcoin_service_registry::{
    ActorService, EventHandler, ServiceContext, ServiceFactory, ServiceRef,
};
use starcoin_txpool_api::PropagateNewTransactions;
use starcoin_types::peer_info::PeerId;
use starcoin_types::peer_info::PeerInfo;
use std::sync::Arc;

//TODO Service registry should support custom service ref.
#[derive(Clone)]
pub struct NetworkServiceRef {
    //hold a network_p2p's network_service for directly send message to NetworkWorker.
    network_service: Arc<network_p2p::NetworkService>,
    service_ref: ServiceRef<NetworkActorService>,
}

impl NetworkService for NetworkServiceRef {
    fn send_peer_message(&self, msg: PeerMessage) {
        self.service_ref.send_peer_message(msg)
    }

    fn broadcast(&self, notification: NotificationMessage) {
        self.service_ref.broadcast(notification)
    }

    fn report_peer(&self, peer_id: PeerId, cost_benefit: ReputationChange) {
        self.service_ref.report_peer(peer_id, cost_benefit)
    }
}

impl PeerProvider for NetworkServiceRef {
    fn peer_set(&self) -> BoxFuture<Result<Vec<PeerInfo>>> {
        self.service_ref.peer_set()
    }

    fn get_peer(&self, peer_id: PeerId) -> BoxFuture<Result<Option<PeerInfo>>> {
        self.service_ref.get_peer(peer_id)
    }

    fn get_self_peer(&self) -> BoxFuture<'_, Result<PeerInfo>> {
        self.service_ref.get_self_peer()
    }
}

impl RawRpcClient for NetworkServiceRef {
    fn send_raw_request(
        &self,
        peer_id: PeerId,
        rpc_path: String,
        message: Vec<u8>,
    ) -> BoxFuture<Result<Vec<u8>>> {
        let protocol = format!("{}{}", RPC_PROTOCOL_PREFIX, rpc_path);
        self.network_service
            .request(peer_id.into(), protocol, message)
            .map_err(|e| e.into())
            .boxed()
    }
}

impl NetworkServiceRef {
    pub fn new(
        network_service: Arc<network_p2p::NetworkService>,
        service_ref: ServiceRef<NetworkActorService>,
    ) -> Self {
        Self {
            network_service,
            service_ref,
        }
    }
    pub fn add_peer(&self, peer: String) -> Result<()> {
        self.network_service
            .add_reserved_peer(peer)
            .map_err(|e| format_err!("{:?}", e))
    }

    pub async fn network_state(&self) -> Result<NetworkState> {
        self.network_service
            .network_state()
            .await
            .map_err(|_| format_err!("request cancel."))
    }

    pub async fn known_peers(&self) -> Vec<PeerId> {
        self.network_service
            .known_peers()
            .await
            .into_iter()
            .map(|peer_id| peer_id.into())
            .collect()
    }

    pub async fn get_address(&self, peer_id: PeerId) -> Vec<Multiaddr> {
        self.network_service.get_address(peer_id.into()).await
    }

    // pub fn start<H>(
    //     node_config: Arc<NodeConfig>,
    //     chain_info: ChainInfo,
    //     bus: ServiceRef<BusService>,
    //     rpc_info: RpcInfo,
    //     network_rpc_service: ServiceRef<NetworkRpcService>,
    //     peer_message_handler: H,
    // ) -> Result<NetworkAsyncService>
    // where
    //     H: PeerMessageHandler + 'static,
    // {
    //     let peer_id = node_config.network.self_peer_id()?;
    //
    //     let self_info = PeerInfo::new(peer_id, chain_info.clone());
    //
    //     // merge seeds from chain config
    //     let mut config = node_config.network.clone();
    //     if !node_config.network.disable_seed {
    //         let seeds = node_config.net().boot_nodes().to_vec();
    //         config.seeds.extend(seeds);
    //     }
    //     let has_seed = !config.seeds.is_empty();
    //
    //     let (service, tx, rx, event_rx, tx_command) = build_network_service(
    //         node_config.node_name().to_string(),
    //         chain_info,
    //         &config,
    //         NotificationMessage::protocols(),
    //         Some((rpc_info, network_rpc_service)),
    //     );
    //     info!(
    //         "network started at {} with seed {},network address is {}",
    //         &node_config.network.listen,
    //         &node_config
    //             .network
    //             .seeds
    //             .iter()
    //             .fold(String::new(), |acc, arg| acc + arg.to_string().as_str()),
    //         service.identify()
    //     );
    //
    //     let (connected_tx, mut connected_rx) = futures::channel::mpsc::channel(1);
    //     let need_send_event = AtomicBool::new(false);
    //
    //     if has_seed && !node_config.network.disable_seed {
    //         need_send_event.swap(true, Ordering::Acquire);
    //     }
    //
    //     let metrics = NetworkMetrics::register().ok();
    //
    //     let inner = Inner {
    //         network_service: service,
    //         self_peer_info: Peer::new(self_info),
    //         bus,
    //         peers: Arc::new(Mutex::new(HashMap::new())),
    //         connected_tx,
    //         need_send_event,
    //         peer_message_handler: Arc::new(peer_message_handler),
    //         metrics,
    //     };
    //     let inner = Arc::new(inner);
    //
    //     // TODO: unify all async runtimes into one.
    //     async_std::task::spawn(Inner::start(inner.clone(), rx, event_rx, tx_command));
    //
    //     if has_seed {
    //         info!("Seed was in configuration and not ignored.So wait for connection open event.");
    //         futures::executor::block_on(async move {
    //             if let Some(event) = connected_rx.next().await {
    //                 info!("Receive event {:?}, network started.", event);
    //             } else {
    //                 error!("Wait peer event return None.");
    //             }
    //         });
    //     }
    //
    //     Ok(NetworkAsyncService { tx, inner })
    // }
}

// TODO: figure out a better place for the actor.
/// Used to manage broadcast new txn to other network peers.
pub struct PeerMsgBroadcasterService {
    network: NetworkServiceRef,
}

impl PeerMsgBroadcasterService {
    pub fn new(network: NetworkServiceRef) -> Self {
        Self { network }
    }
}

impl ServiceFactory<Self> for PeerMsgBroadcasterService {
    fn create(
        ctx: &mut ServiceContext<PeerMsgBroadcasterService>,
    ) -> Result<PeerMsgBroadcasterService> {
        let network = ctx.get_shared::<NetworkServiceRef>()?;
        Ok(Self::new(network))
    }
}

impl ActorService for PeerMsgBroadcasterService {
    fn started(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.subscribe::<PropagateNewTransactions>();
        Ok(())
    }

    fn stopped(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.unsubscribe::<PropagateNewTransactions>();
        Ok(())
    }
}

// handle txn relayer
impl EventHandler<Self, PropagateNewTransactions> for PeerMsgBroadcasterService {
    fn handle_event(
        &mut self,
        msg: PropagateNewTransactions,
        _ctx: &mut ServiceContext<PeerMsgBroadcasterService>,
    ) {
        let txns = msg.propagate_transaction();
        if txns.is_empty() {
            error!("broadcast PropagateNewTransactions is empty.");
            return;
        }
        debug!("propagate new txns, len: {}", txns.len());
        self.network
            .broadcast(NotificationMessage::Transactions(TransactionsMessage::new(
                txns,
            )));
    }
}
