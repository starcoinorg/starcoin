pub mod event;
pub mod generic_proto;
pub mod message;

use crate::protocol::generic_proto::{GenericProto, GenericProtoOut, NotificationsSink};
use crate::protocol::message::generic::Status;
use crate::utils::interval;
use crate::{errors, DiscoveryNetBehaviour, Multiaddr};
use bcs_ext::BCSCodec;
use bytes::Bytes;
use futures::prelude::*;
use libp2p::core::{
    connection::{ConnectionId, ListenerId},
    ConnectedPoint,
};
use libp2p::swarm::{IntoProtocolsHandler, ProtocolsHandler};
use libp2p::swarm::{NetworkBehaviour, NetworkBehaviourAction, PollParameters};
use libp2p::PeerId;
use log::Level;
use sc_peerset::{peersstate::PeersState, SetId};
use starcoin_types::startup_info::{ChainInfo, ChainStatus};
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::fmt::Write;
use std::pin::Pin;
use std::str;
use std::sync::Arc;
use std::task::Poll;
use std::time;
use std::time::Duration;

//const REQUEST_TIMEOUT_SEC: u64 = 40;
/// Interval at which we perform time based maintenance
const TICK_TIMEOUT: time::Duration = time::Duration::from_millis(1100);
/// Current protocol version.
pub(crate) const CURRENT_VERSION: u32 = 5;
/// Lowest version we support
pub(crate) const MIN_VERSION: u32 = 3;

pub(crate) const HARD_CORE_PROTOCOL_ID: sc_peerset::SetId = sc_peerset::SetId::from(0);

pub mod rep {
    use sc_peerset::ReputationChange as Rep;

    /// Reputation change when a peer is "clogged", meaning that it's not fast enough to process our
    /// messages.
    pub const CLOGGED_PEER: Rep = Rep::new(-(1 << 12), "Clogged message queue");
    /// Reputation change when a peer doesn't respond in time to our messages.
    pub const TIMEOUT: Rep = Rep::new(-(1 << 10), "Request timeout");
    /// Reputation change when a peer sends us a status message while we already received one.
    pub const UNEXPECTED_STATUS: Rep = Rep::new(-(1 << 20), "Unexpected status message");
    /// We received a message that failed to decode.
    pub const BAD_MESSAGE: Rep = Rep::new(-(1 << 12), "Bad message");
    /// Peer has different genesis.
    pub const GENESIS_MISMATCH: Rep = Rep::new_fatal("Genesis mismatch");
    /// Peer is on unsupported protocol version.
    pub const BAD_PROTOCOL: Rep = Rep::new_fatal("Unsupported protocol");
}

#[derive(Debug)]
pub enum CustomMessageOutcome {
    /// Notification protocols have been opened with a remote.
    NotificationStreamOpened {
        remote: PeerId,
        protocol: Cow<'static, str>,
        notifications_sink: NotificationsSink,
        info: Box<ChainInfo>,
        notif_protocols: Vec<Cow<'static, str>>,
        rpc_protocols: Vec<Cow<'static, str>>,
    },
    /// The [`NotificationsSink`] of some notification protocols need an update.
    NotificationStreamReplaced {
        remote: PeerId,
        protocol: Cow<'static, str>,
        notifications_sink: NotificationsSink,
    },
    /// Notification protocols have been closed with a remote.
    NotificationStreamClosed {
        remote: PeerId,
        protocol: Cow<'static, str>,
    },
    /// Messages have been received on one or more notifications protocols.
    NotificationsReceived {
        remote: PeerId,
        messages: Vec<(Cow<'static, str>, Bytes)>,
    },
    None,
    Banned(PeerId, Duration),
}

/// Peer information
#[derive(Debug, Clone)]
struct Peer {
    #[allow(unused)]
    info: ChainInfo,
}

#[derive(Default)]
struct PacketStats {
    bytes_in: u64,
    bytes_out: u64,
    count_in: u64,
    count_out: u64,
}

struct ContextData {
    // All connected peers
    peers: HashMap<PeerId, Peer>,
    stats: HashMap<&'static str, PacketStats>,
}

pub struct Protocol {
    /// Interval at which we call `tick`.
    tick_timeout: Pin<Box<dyn Stream<Item = ()> + Send>>,
    important_peers: HashSet<PeerId>,
    /// Used to report reputation changes.
    peerset_handle: sc_peerset::PeersetHandle,
    /// Handles opening the unique substream and sending and receiving raw messages.
    behaviour: GenericProto,
    context_data: ContextData,
    /// The `PeerId`'s of all boot nodes.
    boot_node_ids: Arc<HashSet<PeerId>>,
    notif_protocols: Vec<Cow<'static, str>>,
    rpc_protocols: Vec<Cow<'static, str>>,
    /// If we receive a new "substream open" event that contains an invalid handshake, we ask the
    /// inner layer to force-close the substream. Force-closing the substream will generate a
    /// "substream closed" event. This is a problem: since we can't propagate the "substream open"
    /// event to the outer layers, we also shouldn't propagate this "substream closed" event. To
    /// solve this, an entry is added to this map whenever an invalid handshake is received.
    /// Entries are removed when the corresponding "substream closed" is later received.
    bad_handshake_substreams: HashSet<(PeerId, sc_peerset::SetId)>,
    chain_info: ChainInfo,
}

impl NetworkBehaviour for Protocol {
    type ProtocolsHandler = <GenericProto as NetworkBehaviour>::ProtocolsHandler;
    type OutEvent = CustomMessageOutcome;

    fn new_handler(&mut self) -> Self::ProtocolsHandler {
        self.behaviour.new_handler()
    }

    fn addresses_of_peer(&mut self, peer_id: &PeerId) -> Vec<Multiaddr> {
        self.behaviour.addresses_of_peer(peer_id)
    }

    fn inject_connected(&mut self, peer_id: &PeerId) {
        self.behaviour.inject_connected(peer_id)
    }

    fn inject_disconnected(&mut self, peer_id: &PeerId) {
        self.behaviour.inject_disconnected(peer_id)
    }

    fn inject_connection_established(
        &mut self,
        peer_id: &PeerId,
        conn: &ConnectionId,
        endpoint: &ConnectedPoint,
        failed_addresses: Option<&Vec<Multiaddr>>,
    ) {
        self.behaviour
            .inject_connection_established(peer_id, conn, endpoint, failed_addresses)
    }

    fn inject_connection_closed(
        &mut self,
        peer_id: &PeerId,
        conn: &ConnectionId,
        endpoint: &ConnectedPoint,
        handler: <Self::ProtocolsHandler as IntoProtocolsHandler>::Handler,
    ) {
        self.behaviour
            .inject_connection_closed(peer_id, conn, endpoint, handler)
    }

    fn inject_event(
        &mut self,
        peer_id: PeerId,
        connection: ConnectionId,
        event: <<Self::ProtocolsHandler as IntoProtocolsHandler>::Handler as ProtocolsHandler>::OutEvent,
    ) {
        self.behaviour.inject_event(peer_id, connection, event)
    }

    fn inject_dial_failure(
        &mut self,
        peer_id: Option<PeerId>,
        handler: Self::ProtocolsHandler,
        error: &libp2p::swarm::DialError,
    ) {
        self.behaviour.inject_dial_failure(peer_id, handler, error);
    }

    fn inject_new_listen_addr(&mut self, id: ListenerId, addr: &Multiaddr) {
        self.behaviour.inject_new_listen_addr(id, addr)
    }

    fn inject_expired_listen_addr(&mut self, id: ListenerId, addr: &Multiaddr) {
        self.behaviour.inject_expired_listen_addr(id, addr)
    }

    fn inject_new_external_addr(&mut self, addr: &Multiaddr) {
        self.behaviour.inject_new_external_addr(addr)
    }

    fn inject_listener_error(&mut self, id: ListenerId, err: &(dyn std::error::Error + 'static)) {
        self.behaviour.inject_listener_error(id, err);
    }

    fn inject_listener_closed(&mut self, id: ListenerId, reason: Result<(), &std::io::Error>) {
        self.behaviour.inject_listener_closed(id, reason);
    }

    fn poll(
        &mut self,
        cx: &mut std::task::Context,
        params: &mut impl PollParameters,
    ) -> Poll<NetworkBehaviourAction<Self::OutEvent, Self::ProtocolsHandler>> {
        while let Poll::Ready(Some(())) = self.tick_timeout.poll_next_unpin(cx) {
            self.tick();
        }

        let event = match self.behaviour.poll(cx, params) {
            Poll::Pending => return Poll::Pending,
            Poll::Ready(NetworkBehaviourAction::GenerateEvent(ev)) => ev,
            Poll::Ready(NetworkBehaviourAction::Dial { opts, handler }) => {
                return Poll::Ready(NetworkBehaviourAction::Dial { opts, handler });
            }
            Poll::Ready(NetworkBehaviourAction::NotifyHandler {
                peer_id,
                handler,
                event,
            }) => {
                return Poll::Ready(NetworkBehaviourAction::NotifyHandler {
                    peer_id,
                    handler,
                    event,
                });
            }
            Poll::Ready(NetworkBehaviourAction::ReportObservedAddr { address, score }) => {
                return Poll::Ready(NetworkBehaviourAction::ReportObservedAddr { address, score });
            }
            Poll::Ready(NetworkBehaviourAction::CloseConnection {
                peer_id,
                connection,
            }) => {
                return Poll::Ready(NetworkBehaviourAction::CloseConnection {
                    peer_id,
                    connection,
                });
            }
        };

        let outcome = match event {
            GenericProtoOut::CustomProtocolOpen {
                peer_id,
                set_id,
                received_handshake,
                notifications_sink,
            } => match Status::decode(&received_handshake[..]) {
                Ok(status) => {
                    let protocol_name = self.notif_protocols[usize::from(set_id)].clone();
                    self.on_peer_connected(
                        peer_id,
                        set_id,
                        protocol_name,
                        status,
                        notifications_sink,
                    )
                }
                Err(err) => {
                    error!(target: "network-p2p", "Couldn't decode handshake packet sent by {}: {:?}: {}", peer_id, hex::encode(received_handshake), err);
                    self.bad_handshake_substreams.insert((peer_id, set_id));
                    self.peerset_handle.report_peer(peer_id, rep::BAD_MESSAGE);
                    self.behaviour
                        .disconnect_peer(&peer_id, HARD_CORE_PROTOCOL_ID);
                    CustomMessageOutcome::None
                }
            },
            GenericProtoOut::CustomProtocolClosed { peer_id, set_id } => {
                // TODO: check if disconnect peer
                if self.bad_handshake_substreams.remove(&(peer_id, set_id)) {
                    // The substream that has just been closed had been opened with a bad
                    // handshake. The outer layers have never received an opening event about this
                    // substream, and consequently shouldn't receive a closing event either.
                    CustomMessageOutcome::None
                } else {
                    CustomMessageOutcome::NotificationStreamClosed {
                        remote: peer_id,
                        protocol: self.notif_protocols[usize::from(set_id)].clone(),
                    }
                }
            }
            GenericProtoOut::CustomProtocolReplaced {
                peer_id,
                set_id,
                notifications_sink,
            } => {
                if self.bad_handshake_substreams.contains(&(peer_id, set_id)) {
                    CustomMessageOutcome::None
                } else {
                    CustomMessageOutcome::NotificationStreamReplaced {
                        remote: peer_id,
                        protocol: self.notif_protocols[usize::from(set_id)].clone(),
                        notifications_sink,
                    }
                }
            }
            GenericProtoOut::Notification {
                peer_id,
                set_id,
                message,
            } => {
                let protocol_name = self.notif_protocols[usize::from(set_id)].clone();
                self.on_notify(peer_id, vec![(protocol_name, message.freeze())])
            }
            GenericProtoOut::Banned(peer_id, duration) => {
                CustomMessageOutcome::Banned(peer_id, duration)
            }
        };

        if !matches!(outcome, CustomMessageOutcome::None) {
            return Poll::Ready(NetworkBehaviourAction::GenerateEvent(outcome));
        }
        // This block can only be reached if an event was pulled from the behaviour and that
        // resulted in `CustomMessageOutcome::None`. Since there might be another pending
        // message from the behaviour, the task is scheduled again.
        cx.waker().wake_by_ref();
        Poll::Pending
    }
}

impl DiscoveryNetBehaviour for Protocol {
    fn add_discovered_nodes(&mut self, peer_ids: impl Iterator<Item = PeerId>) {
        for peer_id in peer_ids {
            for (set_id, _) in self.notif_protocols.iter().enumerate() {
                self.peerset_handle
                    .add_to_peers_set(SetId::from(set_id), peer_id);
            }
        }
    }
}

impl Protocol {
    /// Create a new instance.
    pub fn new(
        peerset_config: sc_peerset::PeersetConfig,
        chain_info: ChainInfo,
        boot_node_ids: Arc<HashSet<PeerId>>,
        notif_protocols: Vec<Cow<'static, str>>,
        rpc_protocols: Vec<Cow<'static, str>>,
    ) -> errors::Result<(Protocol, sc_peerset::PeersetHandle)> {
        let mut important_peers = HashSet::new();
        important_peers.extend(boot_node_ids.iter());
        for peer_set in peerset_config.sets.iter() {
            for reserved in &peer_set.reserved_nodes {
                important_peers.insert(*reserved);
            }
        }

        let (peerset, peerset_handle) = sc_peerset::Peerset::from_config(peerset_config);
        let behaviour = {
            let handshake_message = Self::build_handshake_msg(
                notif_protocols.to_vec(),
                rpc_protocols.to_vec(),
                chain_info.clone(),
            );

            let notif_protocol_wth_handshake = notif_protocols
                .clone()
                .into_iter()
                .map(|protocol| (protocol, handshake_message.clone(), u64::max_value()));

            debug!(
                "Handshake message: {}",
                hex::encode(handshake_message.as_slice())
            );

            GenericProto::new(peerset, notif_protocol_wth_handshake)
        };

        let protocol = Protocol {
            tick_timeout: Box::pin(interval(TICK_TIMEOUT)),
            important_peers,
            peerset_handle: peerset_handle.clone(),
            behaviour,
            context_data: ContextData {
                peers: HashMap::new(),
                stats: HashMap::new(),
            },
            chain_info,
            boot_node_ids,
            notif_protocols,
            rpc_protocols,
            bad_handshake_substreams: Default::default(),
        };
        Ok((protocol, peerset_handle))
    }

    /// Returns the list of all the peers we have an open channel to.
    pub fn open_peers(&self) -> impl Iterator<Item = &PeerId> {
        self.behaviour.open_peers()
    }

    /// Returns true if we have a channel open with this node.
    pub fn is_open(&self, peer_id: &PeerId) -> bool {
        self.behaviour.is_open(peer_id, HARD_CORE_PROTOCOL_ID)
    }

    /// Returns the list of all the peers that the peerset currently requests us to be connected to.
    pub fn requested_peers(&self) -> impl Iterator<Item = &PeerId> {
        self.behaviour.requested_peers(HARD_CORE_PROTOCOL_ID)
    }
    /// Adjusts the reputation of a node.
    pub fn report_peer(&self, who: PeerId, reputation: sc_peerset::ReputationChange) {
        self.peerset_handle.report_peer(who, reputation)
    }

    /// Returns the number of discovered nodes that we keep in memory.
    pub fn num_discovered_peers(&self) -> usize {
        self.behaviour.num_discovered_peers()
    }

    /// Disconnects the given peer if we are connected to it.
    pub fn disconnect_peer(&mut self, peer_id: &PeerId, protocol_name: &str) {
        if let Some(position) = self
            .notif_protocols
            .iter()
            .position(|p| *p == protocol_name)
        {
            self.behaviour
                .disconnect_peer(peer_id, sc_peerset::SetId::from(position));
        } else {
            log::warn!(target: "sub-libp2p", "disconnect_peer() with invalid protocol name")
        }
    }

    /// Returns the state of the peerset manager, for debugging purposes.
    pub fn peerset_debug_info(&mut self) -> serde_json::Value {
        self.behaviour.peerset_debug_info()
    }
    pub fn peerset_info(&self) -> PeersState {
        self.behaviour.peerset_info()
    }
    pub fn on_notify(
        &mut self,
        who: PeerId,
        messages: Vec<(Cow<'static, str>, Bytes)>,
    ) -> CustomMessageOutcome {
        CustomMessageOutcome::NotificationsReceived {
            remote: who,
            messages,
        }
    }

    /// Called on the first connection between two peers, after their exchange of handshake.
    fn on_peer_connected(
        &mut self,
        who: PeerId,
        set_id: SetId,
        protocol_name: Cow<'static, str>,
        status: Status,
        notifications_sink: NotificationsSink,
    ) -> CustomMessageOutcome {
        debug!(target: "network-p2p", "New peer {} {:?}", who, status);
        if status.info.genesis_hash() != self.chain_info.genesis_hash() {
            if self.boot_node_ids.contains(&who) {
                error!(
                    target: "network-p2p",
                    "Bootnode with peer id `{}` is on a different chain (our genesis: {} theirs: {})",
                    who,
                    self.chain_info.genesis_hash(),
                    status.info.genesis_hash(),
                );
            } else {
                log!(
                    target: "network-p2p",
                    if self.important_peers.contains(&who) { Level::Warn } else { Level::Debug },
                    "Peer with id `{}` is on different chain (our genesis: {} theirs: {})",
                    who,
                    self.chain_info.genesis_hash(),
                    status.info.genesis_hash(),
                );
            }
            self.peerset_handle.report_peer(who, rep::GENESIS_MISMATCH);
            return CustomMessageOutcome::None;
        }
        if status.version < MIN_VERSION || CURRENT_VERSION < status.min_supported_version {
            log!(
                target: "network-p2p",
                if self.important_peers.contains(&who) { Level::Warn } else { Level::Debug },
                "Peer {:?} using unsupported protocol version {}", who, status.version
            );
            self.peerset_handle.report_peer(who, rep::BAD_PROTOCOL);
            return CustomMessageOutcome::None;
        }
        debug!(target: "network-p2p", "Connected {}", who);
        let peer = Peer {
            info: status.info.clone(),
        };
        self.context_data.peers.insert(who, peer);
        debug!(target: "network-p2p", "Connected {}, Set id {:?}", who, set_id);
        CustomMessageOutcome::NotificationStreamOpened {
            remote: who,
            protocol: protocol_name,
            notifications_sink,
            info: Box::new(status.info),
            notif_protocols: status.notif_protocols.to_vec(),
            rpc_protocols: status.rpc_protocols.to_vec(),
        }
    }

    fn build_status(
        notif_protocols: Vec<Cow<'static, str>>,
        rpc_protocols: Vec<Cow<'static, str>>,
        info: ChainInfo,
    ) -> Status {
        message::generic::Status {
            version: CURRENT_VERSION,
            min_supported_version: MIN_VERSION,
            notif_protocols,
            rpc_protocols,
            info,
        }
    }

    fn build_handshake_msg(
        notif_protocols: Vec<Cow<'static, str>>,
        rpc_protocols: Vec<Cow<'static, str>>,
        info: ChainInfo,
    ) -> Vec<u8> {
        Self::build_status(notif_protocols, rpc_protocols, info)
            .encode()
            .expect("Status encode should success.")
    }

    /// Called by peer when it is disconnecting
    pub fn on_peer_disconnected(
        &mut self,
        peer: PeerId,
        protocol: Cow<'static, str>,
    ) -> CustomMessageOutcome {
        if self.important_peers.contains(&peer) {
            warn!(target: "network-p2p", "Reserved peer {} disconnected", peer);
        } else {
            trace!(target: "network-p2p", "{} disconnected", peer);
        }
        if let Some(_peer_data) = self.context_data.peers.remove(&peer) {
            // Notify all the notification protocols as closed.
            CustomMessageOutcome::NotificationStreamClosed {
                remote: peer,
                protocol,
            }
        } else {
            CustomMessageOutcome::None
        }
    }

    /// Called as a back-pressure mechanism if the networking detects that the peer cannot process
    /// our messaging rate fast enough.
    pub fn on_clogged_peer(&self, who: PeerId) {
        self.peerset_handle.report_peer(who, rep::CLOGGED_PEER);
    }

    /// Perform time based maintenance.
    ///
    /// > **Note**: This method normally doesn't have to be called except for testing purposes.
    pub fn tick(&mut self) {
        self.maintain_peers();
    }

    fn maintain_peers(&mut self) {}

    /// Returns the number of peers we're connected to.
    pub fn num_connected_peers(&self) -> usize {
        self.context_data.peers.values().count()
    }

    pub fn update_chain_status(&mut self, chain_status: ChainStatus) {
        self.chain_info.update_status(chain_status);
        self.update_handshake();
    }

    fn update_handshake(&mut self) {
        let handshake_msg = Self::build_handshake_msg(
            self.notif_protocols.to_vec(),
            self.rpc_protocols.to_vec(),
            self.chain_info.clone(),
        );
        for (set_id, _) in self.notif_protocols.iter().enumerate() {
            self.behaviour
                .set_notif_protocol_handshake(SetId::from(set_id), handshake_msg.clone());
        }
    }

    fn format_stats(&self) -> String {
        let mut out = String::new();
        for (id, stats) in &self.context_data.stats {
            let _ = writeln!(
                &mut out,
                "{}: In: {} bytes ({}), Out: {} bytes ({})",
                id, stats.bytes_in, stats.count_in, stats.bytes_out, stats.count_out,
            );
        }
        out
    }
}

impl Drop for Protocol {
    fn drop(&mut self) {
        debug!(target: "sync", "Network stats:\n{}", self.format_stats());
    }
}
