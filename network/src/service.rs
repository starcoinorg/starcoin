// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::broadcast_score_metrics::BROADCAST_SCORE_METRICS;
use crate::network_metrics::NetworkMetrics;
use crate::{build_network_worker, Announcement};
use anyhow::{format_err, Result};
use bytes::Bytes;
use futures::future::{abortable, AbortHandle};
use futures::FutureExt;
use log::{debug, error, info, trace};
use lru::LruCache;
use network_api::messages::{
    AnnouncementType, GetPeerById, GetPeerSet, GetSelfPeer, NotificationMessage, PeerEvent,
    PeerMessage, PeerReputations, ReportReputation, TransactionsMessage,
};
use network_api::peer_score::{BlockBroadcastEntry, HandleState, LinearScore, Score};
use network_api::{BroadcastProtocolFilter, NetworkActor, PeerMessageHandler};
use network_p2p::{Event, NetworkWorker};
use rand::prelude::SliceRandom;
use starcoin_config::NodeConfig;
use starcoin_crypto::HashValue;
use starcoin_network_rpc::NetworkRpcService;
use starcoin_service_registry::{
    ActorService, EventHandler, ServiceContext, ServiceHandler, ServiceRef, ServiceRequest,
};
use starcoin_txpool_api::PropagateTransactions;
use starcoin_types::peer_info::{PeerId, PeerInfo, RpcInfo};
use starcoin_types::startup_info::{ChainInfo, ChainStatus};
use starcoin_types::sync_status::SyncStatus;
use starcoin_types::system_events::SyncStatusChangeEvent;
use std::borrow::Cow;
use std::collections::HashMap;
use std::ops::RangeInclusive;
use std::sync::Arc;

pub struct NetworkActorService {
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
        let (self_info, worker) = build_network_worker(
            &config.network,
            chain_info,
            config.network.supported_network_protocols(),
            rpc,
        )?;
        let service = worker.service().clone();
        //let self_info = PeerInfo::new(config.network.self_peer_id(), chain_info);
        let inner = Inner::new(config, self_info, service, peer_message_handler)?;
        Ok(Self {
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
        ctx.subscribe::<SyncStatusChangeEvent>();
        ctx.subscribe::<PropagateTransactions>();
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

    fn stopped(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.unsubscribe::<SyncStatusChangeEvent>();
        ctx.unsubscribe::<PropagateTransactions>();
        if let Some(abort_handle) = self.network_worker_handle.take() {
            abort_handle.abort();
        }
        Ok(())
    }
}

impl EventHandler<Self, SyncStatusChangeEvent> for NetworkActorService {
    fn handle_event(&mut self, msg: SyncStatusChangeEvent, _ctx: &mut ServiceContext<Self>) {
        self.inner.update_chain_status(msg.0);
    }
}

impl EventHandler<Self, Event> for NetworkActorService {
    fn handle_event(&mut self, event: Event, ctx: &mut ServiceContext<NetworkActorService>) {
        match event {
            Event::Dht(_) => {
                debug!("ignore dht event");
            }
            Event::NotificationStreamOpened {
                remote,
                protocol,
                info,
                notif_protocols,
                rpc_protocols,
            } => {
                //TODO Refactor PeerEvent for handle protocol and substream.
                // Currently, every notification stream open will trigger a PeerEvent, so it will trigger repeat event.
                debug!(
                    "Connected peer {:?}, protocol: {}, notif_protocols: {:?}, rpc_protocols: {:?}",
                    remote, protocol, notif_protocols, rpc_protocols
                );
                let peer_event = PeerEvent::Open(remote.clone().into(), info.clone());
                self.inner
                    .on_peer_connected(remote.into(), *info, notif_protocols, rpc_protocols);
                ctx.broadcast(peer_event);
            }
            Event::NotificationStreamClosed { remote, .. } => {
                debug!("Close peer {:?}", remote);
                let peer_event = PeerEvent::Close(remote.clone().into());
                self.inner.on_peer_disconnected(remote.into());
                ctx.broadcast(peer_event);
            }
            Event::NotificationsReceived { remote, messages } => {
                for (protocol, message) in messages {
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

// handle txn relayer
impl EventHandler<Self, PropagateTransactions> for NetworkActorService {
    fn handle_event(
        &mut self,
        msg: PropagateTransactions,
        _ctx: &mut ServiceContext<NetworkActorService>,
    ) {
        let txns = msg.transaction_to_propagate();
        if txns.is_empty() {
            return;
        }
        debug!("prepare to propagate txns, len: {}", txns.len());
        self.inner
            .broadcast(NotificationMessage::Transactions(TransactionsMessage::new(
                txns,
            )));
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

impl ServiceHandler<Self, PeerReputations> for NetworkActorService {
    fn handle(
        &mut self,
        msg: PeerReputations,
        ctx: &mut ServiceContext<NetworkActorService>,
    ) -> <PeerReputations as ServiceRequest>::Response {
        let rx = self.inner.network_service.reputations(msg.threshold);
        let fut = async move {
            match rx.await {
                Ok(t) => t
                    .into_iter()
                    .map(|(peer_id, score)| (PeerId::new(peer_id), score))
                    .collect(),
                Err(e) => {
                    debug!("sth wrong {}", e);
                    Vec::new()
                }
            }
        };
        ctx.exec(fut)
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

// max peers is 100(in: 25 + out:75), so blocks lru + txn lru max memory usage about is:
// (100 +1 ) * ( LRU_CACHE_SIZE * 32) *2 = 64M
const LRU_CACHE_SIZE: usize = 10240;

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
    config: Arc<NodeConfig>,
    network_service: Arc<network_p2p::NetworkService>,
    self_peer: Peer,
    peers: HashMap<PeerId, Peer>,
    peer_message_handler: Arc<dyn PeerMessageHandler>,
    metrics: Option<NetworkMetrics>,
    score_handler: Arc<dyn Score<BlockBroadcastEntry> + 'static>,
}

impl BroadcastProtocolFilter for Inner {
    fn peer_info(&self, peer_id: &PeerId) -> Option<PeerInfo> {
        self.peers
            .get(peer_id)
            .map(|peer: &Peer| -> PeerInfo { peer.peer_info.clone() })
    }

    fn is_supported(&self, peer_id: &PeerId, notif_protocol: Cow<'static, str>) -> bool {
        if let Some(peer) = self.peers.get(peer_id) {
            return peer.peer_info.is_support_notif_protocol(notif_protocol);
        }
        false
    }
}

impl Inner {
    pub fn new<H>(
        config: Arc<NodeConfig>,
        self_info: PeerInfo,
        network_service: Arc<network_p2p::NetworkService>,
        peer_message_handler: H,
    ) -> Result<Inner>
    where
        H: PeerMessageHandler + 'static,
    {
        let metrics = NetworkMetrics::register().ok();

        Ok(Inner {
            config,
            network_service,
            self_peer: Peer::new(self_info),
            peers: HashMap::new(),
            peer_message_handler: Arc::new(peer_message_handler),
            metrics,
            score_handler: Arc::new(LinearScore::new(10)),
        })
    }

    pub(crate) fn update_chain_status(&mut self, sync_status: SyncStatus) {
        let chain_status = sync_status.chain_status().clone();
        self.self_peer
            .peer_info
            .update_chain_status(chain_status.clone());
        self.network_service.update_chain_status(chain_status);
    }

    pub(crate) fn handle_network_message(
        &mut self,
        peer_id: PeerId,
        protocol: Cow<'static, str>,
        message: Bytes,
    ) -> Result<()> {
        if let Some(peer_info) = self.peers.get_mut(&peer_id) {
            let notification =
                NotificationMessage::decode_notification(protocol.as_ref(), message.as_ref())?;
            let notification = match &notification {
                NotificationMessage::Transactions(peer_transactions) => {
                    for txn in &peer_transactions.txns {
                        let id = txn.id();
                        peer_info.known_transactions.put(id, ());
                    }
                    let txns_after_filter = peer_transactions
                        .txns
                        .iter()
                        .filter(|txn| {
                            let txn_id = txn.id();
                            if !self.self_peer.known_transactions.contains(&txn_id) {
                                self.self_peer.known_transactions.put(txn_id, ());
                                true
                            } else {
                                false
                            }
                        })
                        .collect::<Vec<_>>();

                    if txns_after_filter.len() == peer_transactions.txns.len() {
                        Some(notification)
                    } else if txns_after_filter.is_empty() {
                        None
                    } else {
                        Some(NotificationMessage::Transactions(TransactionsMessage::new(
                            txns_after_filter.into_iter().cloned().collect(),
                        )))
                    }
                }
                NotificationMessage::CompactBlock(compact_block_message) => {
                    let block_header = compact_block_message.compact_block.header.clone();
                    let total_difficulty = compact_block_message.block_info.total_difficulty;
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
                    peer_info.peer_info.update_chain_status(ChainStatus::new(
                        block_header,
                        compact_block_message.block_info.clone(),
                    ));

                    if self.self_peer.known_blocks.contains(&block_id) {
                        None
                    } else {
                        self.self_peer.known_blocks.put(block_id, ());
                        Some(notification)
                    }
                }
                NotificationMessage::Announcement(announcement) => {
                    debug!("announcement ids length: {:?}", announcement.ids.len());
                    if announcement.is_txn() {
                        let mut fresh_ids = Vec::new();
                        for txn_id in announcement.clone().ids() {
                            peer_info.known_transactions.put(txn_id, ());

                            if !self.self_peer.known_transactions.contains(&txn_id) {
                                self.self_peer.known_transactions.put(txn_id, ());
                                fresh_ids.push(txn_id);
                            };
                        }

                        if fresh_ids.is_empty() {
                            None
                        } else {
                            Some(NotificationMessage::Announcement(Announcement::new(
                                AnnouncementType::Txn,
                                fresh_ids,
                            )))
                        }
                    } else {
                        None
                    }
                }
            };

            if let Some(notification) = notification {
                debug!("notification protocol : {:?}", notification.protocol_name());
                let peer_message = PeerMessage::new(peer_id.clone(), notification);
                self.peer_message_handler.handle_message(peer_message);
                BROADCAST_SCORE_METRICS.report_new(
                    peer_id,
                    self.score_handler
                        .execute(BlockBroadcastEntry::new(true, HandleState::Succ)),
                );
            } else {
                debug!(
                    "Receive repeat message from peer: {}, protocol:{}, ignore.",
                    peer_id, protocol
                );
                BROADCAST_SCORE_METRICS.report_expire(
                    peer_id,
                    self.score_handler
                        .execute(BlockBroadcastEntry::new(false, HandleState::Succ)),
                );
            };
        } else {
            error!(
                "Receive NetworkMessage from unknown peer: {}, protocol: {}",
                peer_id, protocol
            )
        }
        Ok(())
    }

    pub(crate) fn on_peer_connected(
        &mut self,
        peer_id: PeerId,
        chain_info: ChainInfo,
        notif_protocols: Vec<Cow<'static, str>>,
        rpc_protocols: Vec<Cow<'static, str>>,
    ) {
        self.peers
            .entry(peer_id.clone())
            .and_modify(|peer| {
                peer.peer_info
                    .update_chain_status(chain_info.status().clone());
            })
            .or_insert_with(|| {
                Peer::new(PeerInfo::new(
                    peer_id,
                    chain_info,
                    notif_protocols,
                    rpc_protocols,
                ))
            });
    }

    pub(crate) fn on_peer_disconnected(&mut self, peer_id: PeerId) {
        self.peers.remove(&peer_id);
    }

    pub(crate) fn send_peer_message(&mut self, peer_id: PeerId, notification: NotificationMessage) {
        let (protocol_name, data) = notification
            .encode_notification()
            .expect("Encode notification message should ok");
        if !self.is_supported(&peer_id, protocol_name.clone()) {
            debug!(
                "[network]protocol {:?} not supported by peer {:?}",
                protocol_name, peer_id
            );
            return;
        }
        match notification {
            NotificationMessage::Transactions(txn_message) => {
                txn_message.txns.iter().for_each(|txn| {
                    self.self_peer.known_transactions.put(txn.id(), ());
                })
            }
            NotificationMessage::CompactBlock(block) => {
                self.self_peer
                    .known_blocks
                    .put(block.compact_block.header.id(), ());
            }
            NotificationMessage::Announcement(announcement) => {
                if announcement.is_txn() {
                    announcement.ids().into_iter().for_each(|txn_id| {
                        self.self_peer.known_transactions.put(txn_id, ());
                    })
                }
            }
        };
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
                let total_difficulty = msg.block_info.total_difficulty;
                debug!(
                    "update self network chain status, total_difficulty is {}, peer_info is {:?}",
                    total_difficulty, self.self_peer.peer_info
                );
                //Update chain status in two case:
                //1. New Block broadcast
                //2. Sync status change.
                // may be update by repeat message, but can not find a more good way.
                self.network_service.update_chain_status(ChainStatus::new(
                    msg.compact_block.header.clone(),
                    msg.block_info.clone(),
                ));

                self.self_peer.known_blocks.put(id, ());
                let mut send_peer_count: usize = 0;
                let (protocol_name, message) = notification
                    .encode_notification()
                    .expect("Encode notification message should ok");

                let unknown_peer_ids = self
                    .peers
                    .values()
                    .filter(|peer| {
                        if peer.known_blocks.contains(&id) {
                            trace!(
                                "peer({:?}) know this block({:?}), so do not broadcast. ",
                                peer.peer_info.peer_id(),
                                id
                            );
                            false
                        } else {
                            true
                        }
                    })
                    .map(|peer| peer.peer_info.peer_id())
                    .collect::<Vec<_>>();
                let filtered_peer_ids = self.filter(unknown_peer_ids, protocol_name.clone());
                let peers_len = self.peers.len() as u32;

                let selected_peers = select_random_peers(
                    self.config
                        .network
                        .min_peers_to_propagate()
                        .max(peers_len / 2)
                        ..=self.config.network.max_peers_to_propagate().max(peers_len), // use max(max_peers_to_propagate,peers_len) to ensure range [min,max] , max > min.
                    filtered_peer_ids.iter(),
                );

                for peer_id in selected_peers {
                    let peer = self.peers.get_mut(&peer_id).expect("peer should exists");
                    send_peer_count = send_peer_count.saturating_add(1);
                    peer.known_blocks.put(id, ());

                    self.network_service.write_notification(
                        peer_id.into(),
                        protocol_name.clone(),
                        message.clone(),
                    )
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
                msg.txns.iter().for_each(|txn| {
                    self.self_peer.known_transactions.put(txn.id(), ());
                });
                let origin_txn_len = msg.txns.len();
                let mut send_peer_count: usize = 0;
                let selected_peers = select_random_peers(
                    self.config.network.min_peers_to_propagate()
                        ..=self.config.network.max_peers_to_propagate(),
                    self.peers
                        .keys()
                        .filter(|id| self.is_supported(id, protocol_name.clone()))
                        .cloned()
                        .collect::<Vec<_>>()
                        .iter(),
                );
                let peers = self.peers.keys().cloned().collect::<Vec<_>>();
                for peer_id in peers {
                    let is_not_announcement = selected_peers.contains(&peer_id);
                    let peer = self.peers.get_mut(&peer_id).expect("peer should exists");
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

                    // if txn after known_transactions filter is same length with origin, just send origin message for avoid encode data again.
                    let (real_protocol_name, data) =
                        if txns_unhandled.len() == origin_txn_len && is_not_announcement {
                            (protocol_name.clone(), origin_message.clone())
                        } else if is_not_announcement {
                            NotificationMessage::Transactions(TransactionsMessage::new(
                                txns_unhandled.into_iter().cloned().collect(),
                            ))
                            .encode_notification()
                            .expect("Encode notification Transactions message should ok")
                        } else {
                            NotificationMessage::Announcement(Announcement::new(
                                AnnouncementType::Txn,
                                txns_unhandled.into_iter().map(|txn| txn.id()).collect(),
                            ))
                            .encode_notification()
                            .expect("Encode notification Announcement message should ok")
                        };
                    if !is_not_announcement
                        && !self.is_supported(&peer_id, real_protocol_name.clone())
                    {
                        debug!(
                            "[network]remote peer: {:?} not support broadcast protocol :{:?}",
                            peer_id, real_protocol_name
                        );
                        continue;
                    }
                    self.network_service.write_notification(
                        peer_id.into(),
                        real_protocol_name,
                        data,
                    );
                    send_peer_count = send_peer_count.saturating_add(1);
                }
                debug!(
                    "[network] broadcast new {} transactions to {} peers",
                    msg.txns.len(),
                    send_peer_count
                );
            }
            NotificationMessage::Announcement(_msg) => {
                error!("[network] can not broadcast announcement message directly.");
            }
        }
    }
}

fn select_random_peers<'a, P>(peer_num_range: RangeInclusive<u32>, peers: P) -> Vec<PeerId>
where
    P: ExactSizeIterator<Item = &'a PeerId>,
{
    let (min_peers, max_peers) = peer_num_range.into_inner();
    let peers_len = peers.len();
    // take sqrt(x) peers
    let mut count = (peers_len as f64).powf(0.5).round() as u32;
    count = count.min(max_peers).max(min_peers);

    let mut random = rand::thread_rng();
    let mut peer_ids: Vec<_> = peers.cloned().collect();
    peer_ids.shuffle(&mut random);
    peer_ids.truncate(count as usize);
    peer_ids
}

#[cfg(test)]
mod test {
    use crate::service::select_random_peers;
    use network_api::PeerId;

    fn create_peers(n: u32) -> Vec<PeerId> {
        (0..n).map(|_| PeerId::random()).collect()
    }

    #[test]
    fn test_select_peer() {
        assert_eq!(select_random_peers(1..=3, create_peers(2).iter()).len(), 1);
        assert_eq!(select_random_peers(2..=5, create_peers(9).iter()).len(), 3);
        assert_eq!(
            select_random_peers(8..=128, create_peers(3).iter()).len(),
            3
        );
        assert_eq!(
            select_random_peers(8..=128, create_peers(4).iter()).len(),
            4
        );
        assert_eq!(
            select_random_peers(8..=128, create_peers(10).iter()).len(),
            8
        );
        assert_eq!(
            select_random_peers(8..=128, create_peers(25).iter()).len(),
            8
        );
        assert_eq!(
            select_random_peers(8..=128, create_peers(64).iter()).len(),
            8
        );
        assert_eq!(select_random_peers(3..=3, create_peers(3).iter()).len(), 3);
    }
}
