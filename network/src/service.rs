// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::build_network_worker;
use crate::network_metrics::NetworkMetrics;
use crate::service_ref::{Inner, Peer};
use anyhow::{format_err, Result};
use bytes::Bytes;
use config::NodeConfig;
use futures::future::{abortable, AbortHandle};
use futures::FutureExt;
use lru::LruCache;
use network_api::messages::{
    GetPeerById, GetPeerSet, GetSelfPeer, NotificationMessage, PeerEvent, PeerMessage,
    ReportReputation, TransactionsMessage,
};
use network_api::{NetworkActor, PeerMessageHandler};
use network_p2p::{Event, NetworkConfiguration, NetworkService, NetworkWorker, ProtocolId};
use smallvec::alloc::borrow::Cow;
use starcoin_crypto::HashValue;
use starcoin_metrics::Registry;
use starcoin_network_rpc::NetworkRpcService;
use starcoin_service_registry::{
    ActorService, EventHandler, ServiceContext, ServiceHandler, ServiceRef, ServiceRequest,
};
use starcoin_types::peer_info::{PeerId, PeerInfo, RpcInfo};
use starcoin_types::startup_info::{ChainInfo, ChainStatus};
use std::collections::HashMap;
use std::sync::Arc;

pub struct NetworkActorService {
    config: Arc<NodeConfig>,
    worker: Option<NetworkWorker>,
    inner: Inner,
    network_worker_handle: Option<AbortHandle>,
}

impl NetworkActor for NetworkActorService {}

impl NetworkActorService {
    pub fn new<H>(
        config: Arc<NodeConfig>,
        chain_info: ChainInfo,
        rpc: Option<(RpcInfo, ServiceRef<NetworkRpcService>)>,
        peer_message_handler: H,
    ) -> Result<Self>
    where
        H: PeerMessageHandler + 'static,
    {
        let worker = build_network_worker(
            config.as_ref(),
            chain_info.clone(),
            NotificationMessage::protocols(),
            rpc,
        )?;
        let service = worker.service().clone();
        let self_info = PeerInfo::new(config.network.self_peer_id()?, chain_info);
        let inner = Inner::new(self_info, service, peer_message_handler)?;
        Ok(Self {
            config,
            worker: Some(worker),
            inner,
            network_worker_handle: None,
        })
    }

    pub fn network_service(&self) -> Arc<network_p2p::NetworkService> {
        self.inner.network_service.clone()
    }
}

impl ActorService for NetworkActorService {
    fn started(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        let worker = self
            .worker
            .take()
            .ok_or_else(|| format_err!("Network worker should init before started."))?;
        let event_stream = self.inner.network_service.event_stream("network");
        ctx.add_stream(event_stream);
        let (fut, abort_handle) = abortable(worker);
        self.network_worker_handle = Some(abort_handle);
        ctx.spawn(fut.then(|result| async {
            match result {
                Err(_abort) => info!("Network worker stopped."),
                Ok(Err(e)) => error!("Network worker unexpect stopped for : {:?}", e),
                Ok(Ok(_)) => {}
            }
        }));
        Ok(())
    }

    fn stopped(&mut self, _ctx: &mut ServiceContext<Self>) -> Result<()> {
        if let Some(abort_handle) = self.network_worker_handle.take() {
            abort_handle.abort();
        }
        Ok(())
    }
}

impl EventHandler<Self, Event> for NetworkActorService {
    fn handle_event(&mut self, event: Event, ctx: &mut ServiceContext<NetworkActorService>) {
        match event {
            Event::Dht(_) => {
                debug!("ignore dht event");
            }
            Event::NotificationStreamOpened { remote, info } => {
                debug!("Connected peer {:?}", remote);
                let peer_event = PeerEvent::Open(remote.clone().into(), info.clone());
                self.inner.on_peer_connected(remote.into(), *info);
                ctx.broadcast(peer_event);
            }
            Event::NotificationStreamClosed { remote } => {
                debug!("Close peer {:?}", remote);
                let peer_event = PeerEvent::Close(remote.clone().into());
                self.inner.on_peer_disconnected(remote.into());
                ctx.broadcast(peer_event);
            }
            Event::NotificationsReceived {
                remote,
                protocol,
                messages,
            } => {
                for message in messages {
                    if let Err(e) = self.inner.handle_network_message(
                        remote.clone().into(),
                        protocol.clone(),
                        message,
                    ) {
                        error!(
                            "Handle network message fail, remote:{}, protocol:{}, error: {:?}",
                            remote, protocol, e
                        )
                    }
                }
            }
        }
    }
}

impl EventHandler<Self, ReportReputation> for NetworkActorService {
    fn handle_event(
        &mut self,
        msg: ReportReputation,
        _ctx: &mut ServiceContext<NetworkActorService>,
    ) {
        self.inner
            .network_service
            .report_peer(msg.peer_id.into(), msg.change);
    }
}

impl EventHandler<Self, NotificationMessage> for NetworkActorService {
    fn handle_event(
        &mut self,
        msg: NotificationMessage,
        _ctx: &mut ServiceContext<NetworkActorService>,
    ) {
        self.inner.broadcast(msg);
    }
}

impl EventHandler<Self, PeerMessage> for NetworkActorService {
    fn handle_event(&mut self, msg: PeerMessage, _ctx: &mut ServiceContext<NetworkActorService>) {
        self.inner.send_peer_message(msg.peer_id, msg.notification);
    }
}

impl ServiceHandler<Self, GetPeerSet> for NetworkActorService {
    fn handle(
        &mut self,
        _msg: GetPeerSet,
        _ctx: &mut ServiceContext<NetworkActorService>,
    ) -> <GetPeerSet as ServiceRequest>::Response {
        self.inner
            .peers
            .iter()
            .map(|(_, peer)| peer.get_peer_info().clone())
            .collect::<Vec<_>>()
    }
}

impl ServiceHandler<Self, GetPeerById> for NetworkActorService {
    fn handle(
        &mut self,
        msg: GetPeerById,
        _ctx: &mut ServiceContext<NetworkActorService>,
    ) -> <GetPeerById as ServiceRequest>::Response {
        self.inner
            .peers
            .get(&msg.peer_id)
            .map(|peer| peer.get_peer_info().clone())
    }
}

impl ServiceHandler<Self, GetSelfPeer> for NetworkActorService {
    fn handle(
        &mut self,
        _msg: GetSelfPeer,
        _ctx: &mut ServiceContext<NetworkActorService>,
    ) -> <GetSelfPeer as ServiceRequest>::Response {
        self.inner.self_peer.get_peer_info().clone()
    }
}

const LRU_CACHE_SIZE: usize = 1024;

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

pub(crate) struct Inner {
    network_service: Arc<network_p2p::NetworkService>,
    self_peer: Peer,
    peers: HashMap<PeerId, Peer>,
    peer_message_handler: Arc<dyn PeerMessageHandler>,
    metrics: Option<NetworkMetrics>,
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
