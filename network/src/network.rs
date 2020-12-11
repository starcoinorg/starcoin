// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::net::{build_network_worker, RPC_PROTOCOL_PREFIX};
use crate::network_metrics::NetworkMetrics;
use crate::service::NetworkActorService;
use crate::{NetworkMessage, PeerEvent, PeerMessage};
use anyhow::{format_err, Result};
use async_trait::async_trait;
use bytes::Bytes;
use config::NodeConfig;
use futures::future::BoxFuture;
use futures::lock::Mutex;
use futures::{channel::mpsc, sink::SinkExt, stream::StreamExt};
use futures::{FutureExt, TryFutureExt};
use lru::LruCache;
use network_api::messages::NotificationMessage;
use network_api::{
    messages::TransactionsMessage, NetworkService, PeerMessageHandler, PeerProvider,
    ReputationChange,
};
use network_p2p_types::network_state::NetworkState;
use network_p2p_types::Multiaddr;
use network_rpc_core::RawRpcClient;
use starcoin_crypto::HashValue;
use starcoin_network_rpc::NetworkRpcService;
use starcoin_service_registry::bus::{Bus, BusService};
use starcoin_service_registry::{
    ActorService, EventHandler, ServiceContext, ServiceFactory, ServiceRef,
};
use starcoin_txpool_api::PropagateNewTransactions;
use starcoin_types::peer_info::PeerId;
use starcoin_types::peer_info::{PeerInfo, RpcInfo};
use starcoin_types::startup_info::{ChainInfo, ChainStatus};
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

const LRU_CACHE_SIZE: usize = 1024;

//TODO rename this service
#[derive(Clone)]
pub struct NetworkAsyncService {
    //hold a network_p2p's network_service for directly send message to NetworkWorker.
    network_service: Arc<network_p2p::NetworkService>,
    service_ref: ServiceRef<NetworkActorService>,
}

pub(crate) struct Inner {
    //network_service: SNetworkService,
    pub(crate) network_service: Arc<network_p2p::NetworkService>,
    pub(crate) self_peer: Peer,
    pub(crate) peers: HashMap<PeerId, Peer>,
    peer_message_handler: Arc<dyn PeerMessageHandler>,
    metrics: Option<NetworkMetrics>,
}

#[derive(Debug)]
pub struct Peer {
    peer_info: PeerInfo,
    known_transactions: LruCache<HashValue, ()>,
    /// Holds a set of blocks known to this peer.
    known_blocks: LruCache<HashValue, ()>,
}

impl Peer {
    fn new(peer_info: PeerInfo) -> Self {
        Self {
            peer_info,
            known_blocks: LruCache::new(LRU_CACHE_SIZE),
            known_transactions: LruCache::new(LRU_CACHE_SIZE),
        }
    }

    pub fn get_peer_info(&self) -> &PeerInfo {
        &self.peer_info
    }
}

impl NetworkService for NetworkAsyncService {
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

impl PeerProvider for NetworkAsyncService {
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

impl RawRpcClient for NetworkAsyncService {
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

impl NetworkAsyncService {
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

impl Inner {
    pub fn new<H>(
        self_info: PeerInfo,
        network_service: Arc<network_p2p::NetworkService>,
        peer_message_handler: H,
    ) -> Result<Inner>
    where
        H: PeerMessageHandler + 'static,
    {
        let metrics = NetworkMetrics::register().ok();

        Ok(Inner {
            network_service,
            self_peer: Peer::new(self_info),
            peers: HashMap::new(),
            peer_message_handler: Arc::new(peer_message_handler),
            metrics,
        })
    }

    pub(crate) fn handle_network_message(
        &mut self,
        peer_id: PeerId,
        protocol: Cow<'static, str>,
        message: Bytes,
    ) -> Result<()> {
        if let Some(peer_info) = self.peers.get_mut(&peer_id) {
            let notification =
                NotificationMessage::decode_notification(protocol, message.as_ref())?;
            match &notification {
                NotificationMessage::Transactions(peer_transactions) => {
                    for txn in &peer_transactions.txns {
                        let id = txn.id();
                        peer_info.known_transactions.put(id, ());
                    }
                }
                NotificationMessage::CompactBlock(compact_block_message) => {
                    let block_header = compact_block_message.compact_block.header.clone();
                    let total_difficulty = compact_block_message.total_difficulty;
                    let block_id = block_header.id();
                    debug!(
                        "Receive new compact block from {:?} with hash {:?}",
                        peer_id, block_id
                    );
                    debug!(
                        "total_difficulty is {},peer_info is {:?}",
                        total_difficulty, peer_info
                    );
                    peer_info.known_blocks.put(block_id, ());
                    peer_info
                        .peer_info
                        .update_chain_status(ChainStatus::new(block_header, total_difficulty));
                }
            }

            let peer_message = PeerMessage::new(peer_id.into(), notification);
            self.peer_message_handler.handle_message(peer_message);
        } else {
            error!(
                "Receive NetworkMessage from unknown peer {}, protocol: {}",
                peer_id, protocol
            )
        }
        Ok(())
    }

    pub(crate) fn on_peer_connected(&mut self, peer_id: PeerId, chain_info: ChainInfo) {
        self.peers
            .entry(peer_id.clone())
            .or_insert_with(|| Peer::new(PeerInfo::new(peer_id.into(), chain_info)));
    }

    pub(crate) fn on_peer_disconnected(&mut self, peer_id: PeerId) {
        self.peers.remove(&peer_id);
    }

    pub(crate) fn send_peer_message(&self, peer_id: PeerId, notification: NotificationMessage) {
        let (protocol_name, data) = notification
            .encode_notification()
            .expect("Encode notification message should ok");
        self.network_service
            .write_notification(peer_id.into(), protocol_name, data);
    }

    pub(crate) fn broadcast(&mut self, notification: NotificationMessage) {
        let _timer = self.metrics.as_ref().map(|metrics| {
            metrics
                .broadcast_duration
                .with_label_values(&[notification.protocol_name().as_ref()])
                .start_timer()
        });

        match &notification {
            NotificationMessage::CompactBlock(msg) => {
                let id = msg.compact_block.header.id();
                let block_header = msg.compact_block.header.clone();
                let total_difficulty = msg.total_difficulty;
                let chain_status = ChainStatus::new(block_header.clone(), total_difficulty);
                debug!(
                    "update self network chain status, total_difficulty is {}, peer_info is {:?}",
                    total_difficulty, self.self_peer.peer_info
                );

                self.self_peer
                    .peer_info
                    .update_chain_status(chain_status.clone());
                self.network_service.update_chain_status(chain_status);
                let mut send_peer_count: usize = 0;
                let (protocol_name, message) = notification
                    .encode_notification()
                    .expect("Encode notification message should ok");
                for (peer_id, peer) in &mut self.peers {
                    if peer.known_blocks.contains(&id)
                        || peer.peer_info.total_difficulty() >= total_difficulty
                    {
                        debug!("peer({:?})'s total_difficulty is >= block({:?})'s total_difficulty or it know this block, so do not broadcast. ", peer_id, id);
                    } else {
                        send_peer_count += 1;
                        peer.known_blocks.put(id, ());

                        self.network_service.write_notification(
                            peer_id.clone().into(),
                            protocol_name.clone(),
                            message.clone(),
                        )
                    }
                }

                debug!(
                    "[network] broadcast new compact block message {:?} to {} peers",
                    id, send_peer_count
                );
            }
            NotificationMessage::Transactions(msg) => {
                let (protocol_name, origin_message) = notification
                    .encode_notification()
                    .expect("Encode notification message should ok");
                let origin_txn_len = msg.txns.len();
                let mut send_peer_count: usize = 0;
                for (peer_id, peer) in &mut self.peers {
                    let txns_unhandled = msg
                        .txns
                        .iter()
                        .filter(|txn| {
                            let id = txn.id();
                            if !peer.known_transactions.contains(&id) {
                                peer.known_transactions.put(id, ());
                                true
                            } else {
                                false
                            }
                        })
                        .collect::<Vec<_>>();

                    if txns_unhandled.is_empty() {
                        debug!(
                            "{} known all transactions, ignore broadcast message.",
                            peer_id
                        );
                        continue;
                    }
                    send_peer_count += 1;
                    // if txn after known_transactions filter is same length with origin, just send origin message for avoid encode data again.
                    if txns_unhandled.len() == origin_txn_len {
                        self.network_service.write_notification(
                            peer_id.clone().into(),
                            protocol_name.clone(),
                            origin_message.clone(),
                        )
                    } else {
                        let notification_after_filter = NotificationMessage::Transactions(
                            TransactionsMessage::new(txns_unhandled.into_iter().cloned().collect()),
                        );
                        let (protocol_name, data) = notification_after_filter
                            .encode_notification()
                            .expect("Encode notification message should ok");
                        self.network_service.write_notification(
                            peer_id.clone().into(),
                            protocol_name,
                            data,
                        );
                    }
                }
                debug!(
                    "[network] broadcast new {} transactions to {} peers",
                    msg.txns.len(),
                    send_peer_count
                );
            }
        }
    }
}

// TODO: figure out a better place for the actor.
/// Used to manage broadcast new txn to other network peers.
pub struct PeerMsgBroadcasterService {
    network: NetworkAsyncService,
}

impl PeerMsgBroadcasterService {
    pub fn new(network: NetworkAsyncService) -> Self {
        Self { network }
    }
}

impl ServiceFactory<Self> for PeerMsgBroadcasterService {
    fn create(
        ctx: &mut ServiceContext<PeerMsgBroadcasterService>,
    ) -> Result<PeerMsgBroadcasterService> {
        let network = ctx.get_shared::<NetworkAsyncService>()?;
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
