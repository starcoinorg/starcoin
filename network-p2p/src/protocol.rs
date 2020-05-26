pub mod event;
pub mod generic_proto;
pub mod message;

use crate::config::ProtocolId;
use crate::protocol::generic_proto::{GenericProto, GenericProtoOut};
use crate::utils::interval;
use crate::{DiscoveryNetBehaviour, Multiaddr};

use crate::network_state::Peer;

use bytes::{Bytes, BytesMut};
use futures::prelude::*;
use libp2p::core::{
    connection::{ConnectionId, ListenerId},
    ConnectedPoint,
};
use libp2p::swarm::{IntoProtocolsHandler, ProtocolsHandler};
use libp2p::swarm::{NetworkBehaviour, NetworkBehaviourAction, PollParameters};
use libp2p::PeerId;
use log::Level;

use crate::protocol::message::generic::{ConsensusMessage, Message, Status};
use crypto::HashValue;
use scs::SCSCodec;
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::pin::Pin;
use std::str;
use std::sync::Arc;
use std::task::Poll;
use std::time;
use types::peer_info::PeerInfo;
use wasm_timer::Instant;

const REQUEST_TIMEOUT_SEC: u64 = 40;
/// Interval at which we perform time based maintenance
const TICK_TIMEOUT: time::Duration = time::Duration::from_millis(1100);
/// Current protocol version.
pub(crate) const CURRENT_VERSION: u32 = 1;
/// Lowest version we support
pub(crate) const MIN_VERSION: u32 = 1;

pub use generic_proto::LegacyConnectionKillError;

mod rep {
    use peerset::ReputationChange as Rep;
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
    NotificationStreamOpened {
        remote: PeerId,
        info: Box<PeerInfo>,
    },
    /// Notification protocols have been closed with a remote.
    NotificationStreamClosed {
        remote: PeerId,
    },
    /// Messages have been received on one or more notifications protocols.
    NotificationsReceived {
        remote: PeerId,
        messages: Vec<Bytes>,
    },
    None,
}

/// A peer that we are connected to
/// and from whom we have not yet received a Status message.
struct HandshakingPeer {
    timestamp: Instant,
}

struct ContextData {
    // All connected peers
    peers: HashMap<PeerId, Peer>,
}

pub struct ChainInfo {
    pub genesis_hash: HashValue,
    pub self_info: PeerInfo,
}

pub struct Protocol {
    /// Interval at which we call `tick`.
    tick_timeout: Pin<Box<dyn Stream<Item = ()> + Send>>,
    important_peers: HashSet<PeerId>,
    /// Connected peers pending Status message.
    handshaking_peers: HashMap<PeerId, HandshakingPeer>,
    /// Used to report reputation changes.
    peerset_handle: peerset::PeersetHandle,
    /// Handles opening the unique substream and sending and receiving raw messages.
    behaviour: GenericProto,
    context_data: ContextData,
    /// The `PeerId`'s of all boot nodes.
    boot_node_ids: Arc<HashSet<PeerId>>,

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
    ) {
        self.behaviour
            .inject_connection_established(peer_id, conn, endpoint)
    }

    fn inject_connection_closed(
        &mut self,
        peer_id: &PeerId,
        conn: &ConnectionId,
        endpoint: &ConnectedPoint,
    ) {
        self.behaviour
            .inject_connection_closed(peer_id, conn, endpoint)
    }

    fn inject_event(
        &mut self,
        peer_id: PeerId,
        connection: ConnectionId,
        event: <<Self::ProtocolsHandler as IntoProtocolsHandler>::Handler as ProtocolsHandler>::OutEvent,
    ) {
        self.behaviour.inject_event(peer_id, connection, event)
    }

    fn poll(
        &mut self,
        cx: &mut std::task::Context,
        params: &mut impl PollParameters,
    ) -> Poll<
        NetworkBehaviourAction<
            <<Self::ProtocolsHandler as IntoProtocolsHandler>::Handler as ProtocolsHandler>::InEvent,
            Self::OutEvent
        >
>{
        while let Poll::Ready(Some(())) = self.tick_timeout.poll_next_unpin(cx) {
            self.tick();
        }

        let event = match self.behaviour.poll(cx, params) {
            Poll::Pending => return Poll::Pending,
            Poll::Ready(NetworkBehaviourAction::GenerateEvent(ev)) => ev,
            Poll::Ready(NetworkBehaviourAction::DialAddress { address }) => {
                return Poll::Ready(NetworkBehaviourAction::DialAddress { address })
            }
            Poll::Ready(NetworkBehaviourAction::DialPeer { peer_id, condition }) => {
                return Poll::Ready(NetworkBehaviourAction::DialPeer { peer_id, condition })
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
                })
            }
            Poll::Ready(NetworkBehaviourAction::ReportObservedAddr { address }) => {
                return Poll::Ready(NetworkBehaviourAction::ReportObservedAddr { address })
            }
        };

        let outcome = match event {
            GenericProtoOut::CustomProtocolOpen { peer_id, .. } => {
                self.on_peer_connected(peer_id);
                CustomMessageOutcome::None
            }
            GenericProtoOut::CustomProtocolClosed { peer_id, .. } => {
                self.on_peer_disconnected(peer_id.clone());
                // Notify all the notification protocols as closed.
                CustomMessageOutcome::NotificationStreamClosed { remote: peer_id }
            }
            GenericProtoOut::LegacyMessage { peer_id, message } => {
                self.on_custom_message(peer_id, message)
            }
            GenericProtoOut::Notification {
                peer_id,
                protocol_name: _protocol_name,
                message,
            } => self.on_custom_message(peer_id, message),
            GenericProtoOut::Clogged {
                peer_id,
                messages: _,
            } => {
                self.on_clogged_peer(peer_id);
                CustomMessageOutcome::None
            }
        };

        if let CustomMessageOutcome::None = outcome {
            Poll::Pending
        } else {
            Poll::Ready(NetworkBehaviourAction::GenerateEvent(outcome))
        }
    }

    fn inject_addr_reach_failure(
        &mut self,
        peer_id: Option<&PeerId>,
        addr: &Multiaddr,
        error: &dyn std::error::Error,
    ) {
        self.behaviour
            .inject_addr_reach_failure(peer_id, addr, error)
    }

    fn inject_dial_failure(&mut self, peer_id: &PeerId) {
        self.behaviour.inject_dial_failure(peer_id)
    }

    fn inject_new_listen_addr(&mut self, addr: &Multiaddr) {
        self.behaviour.inject_new_listen_addr(addr)
    }

    fn inject_expired_listen_addr(&mut self, addr: &Multiaddr) {
        self.behaviour.inject_expired_listen_addr(addr)
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
}

impl DiscoveryNetBehaviour for Protocol {
    fn add_discovered_nodes(&mut self, peer_ids: impl Iterator<Item = PeerId>) {
        self.behaviour.add_discovered_nodes(peer_ids)
    }
}

impl Protocol {
    /// Create a new instance.
    pub fn new(
        peerset_config: peerset::PeersetConfig,
        protocol_id: ProtocolId,
        chain_info: ChainInfo,
        boot_node_ids: Arc<HashSet<PeerId>>,
    ) -> crate::net_error::Result<(Protocol, peerset::PeersetHandle)> {
        let important_peers = {
            let mut imp_p = HashSet::new();
            for reserved in &peerset_config.reserved_nodes {
                imp_p.insert(reserved.clone());
            }
            imp_p.shrink_to_fit();
            imp_p
        };

        let (peerset, peerset_handle) = peerset::Peerset::from_config(peerset_config);
        let versions = &((MIN_VERSION as u8)..=(CURRENT_VERSION as u8)).collect::<Vec<u8>>();
        let behaviour = GenericProto::new(protocol_id, versions, peerset, None);

        let protocol = Protocol {
            tick_timeout: Box::pin(interval(TICK_TIMEOUT)),
            handshaking_peers: HashMap::new(),
            important_peers,
            peerset_handle: peerset_handle.clone(),
            behaviour,
            context_data: ContextData {
                peers: HashMap::new(),
            },
            chain_info,
            boot_node_ids,
        };

        Ok((protocol, peerset_handle))
    }

    /// Returns the list of all the peers we have an open channel to.
    pub fn open_peers(&self) -> impl Iterator<Item = &PeerId> {
        self.behaviour.open_peers()
    }

    /// Returns true if we have a channel open with this node.
    pub fn is_open(&self, peer_id: &PeerId) -> bool {
        self.behaviour.is_open(peer_id)
    }

    /// Disconnects the given peer if we are connected to it.
    pub fn disconnect_peer(&mut self, peer_id: &PeerId) {
        self.behaviour.disconnect_peer(peer_id)
    }

    /// Returns true if we try to open protocols with the given peer.
    pub fn is_enabled(&self, peer_id: &PeerId) -> bool {
        self.behaviour.is_enabled(peer_id)
    }

    /// Returns the state of the peerset manager, for debugging purposes.
    pub fn peerset_debug_info(&mut self) -> serde_json::Value {
        self.behaviour.peerset_debug_info()
    }

    pub fn on_custom_message(&mut self, who: PeerId, data: BytesMut) -> CustomMessageOutcome {
        trace!("receive custom message from {} ", who);
        let message = match Message::decode(&data[..]) {
            Ok(message) => message,
            Err(err) => {
                info!(target: "sync", "Couldn't decode packet sent by {}: {:?}: {}", who, data, err);
                self.peerset_handle.report_peer(who, rep::BAD_MESSAGE);
                return CustomMessageOutcome::None;
            }
        };

        match message {
            Message::Consensus(msg) => CustomMessageOutcome::NotificationsReceived {
                remote: who,
                messages: vec![Bytes::from(msg.data)],
            },
            Message::Status(status) => self.on_status_message(who, status),
        }
    }

    /// Called by peer to report status
    fn on_status_message(&mut self, who: PeerId, status: Status) -> CustomMessageOutcome {
        trace!(target: "sync", "New peer {} {:?}", who, status);
        let _protocol_version = {
            if self.context_data.peers.contains_key(&who) {
                log!(
                    target: "sync",
                    if self.important_peers.contains(&who) { Level::Warn } else { Level::Debug },
                    "Unexpected status packet from {}", who
                );
                self.peerset_handle.report_peer(who, rep::UNEXPECTED_STATUS);
                return CustomMessageOutcome::None;
            }
            if status.genesis_hash != self.chain_info.genesis_hash {
                info!(
                    "Peer is on different chain (our genesis: {} theirs: {})",
                    self.chain_info.genesis_hash, status.genesis_hash
                );
                self.peerset_handle
                    .report_peer(who.clone(), rep::GENESIS_MISMATCH);
                self.behaviour.disconnect_peer(&who);

                if self.boot_node_ids.contains(&who) {
                    error!(
                        target: "sync",
                        "Bootnode with peer id `{}` is on a different chain (our genesis: {} theirs: {})",
                        who,
                        self.chain_info.genesis_hash,
                        status.genesis_hash,
                    );
                }

                return CustomMessageOutcome::None;
            }
            if status.version < MIN_VERSION && CURRENT_VERSION < status.min_supported_version {
                log!(
                    target: "sync",
                    if self.important_peers.contains(&who) { Level::Warn } else { Level::Trace },
                    "Peer {:?} using unsupported protocol version {}", who, status.version
                );
                self.peerset_handle
                    .report_peer(who.clone(), rep::BAD_PROTOCOL);
                self.behaviour.disconnect_peer(&who);
                return CustomMessageOutcome::None;
            }

            match self.handshaking_peers.remove(&who) {
                Some(_handshaking) => {}
                None => {
                    error!(target: "sync", "Received status from previously unconnected node {}", who);
                    return CustomMessageOutcome::None;
                }
            };

            debug!(target: "sync", "Connected {}", who);
            status.version
        };
        // Notify all the notification protocols as open.
        CustomMessageOutcome::NotificationStreamOpened {
            remote: who,
            info: Box::new(status.info),
        }
    }

    fn send_message(&mut self, who: &PeerId, message: Message) -> anyhow::Result<()> {
        send_message(&mut self.behaviour, who, message)?;
        Ok(())
    }

    /// Called when a new peer is connected
    pub fn on_peer_connected(&mut self, who: PeerId) {
        debug!(target: "sync", "Connecting {}", who);
        self.handshaking_peers.insert(
            who.clone(),
            HandshakingPeer {
                timestamp: Instant::now(),
            },
        );
        self.send_status(who);
    }

    /// Send Status message
    fn send_status(&mut self, who: PeerId) {
        let status = message::generic::Status {
            version: CURRENT_VERSION,
            min_supported_version: MIN_VERSION,
            genesis_hash: self.chain_info.genesis_hash,
            info: self.chain_info.self_info.clone(),
        };

        self.send_message(&who, Message::Status(status))
            .expect("should succ")
    }

    /// Called by peer when it is disconnecting
    pub fn on_peer_disconnected(&mut self, peer: PeerId) {
        if self.important_peers.contains(&peer) {
            warn!(target: "sync", "Reserved peer {} disconnected", peer);
        } else {
            trace!(target: "sync", "{} disconnected", peer);
        }

        // lock all the the peer lists so that add/remove peer events are in order
        {
            self.handshaking_peers.remove(&peer);
        };
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

    fn maintain_peers(&mut self) {
        let tick = Instant::now();
        let mut aborting = Vec::new();
        {
            for (who, _) in self.handshaking_peers.iter().filter(|(_, handshaking)| {
                (tick - handshaking.timestamp).as_secs() > REQUEST_TIMEOUT_SEC
            }) {
                debug!(
                    target: "sync",
                    "Handshake timeout {}", who
                );
                aborting.push(who.clone());
            }
        }

        for p in aborting {
            self.behaviour.disconnect_peer(&p);
            self.peerset_handle.report_peer(p, rep::TIMEOUT);
        }
    }

    /// Returns the number of peers we're connected to.
    pub fn num_connected_peers(&self) -> usize {
        self.context_data.peers.values().count()
    }

    /// Send a notification to the given peer we're connected to.
    ///
    /// Doesn't do anything if we don't have a notifications substream for that protocol with that
    /// peer.
    pub fn write_notification(
        &mut self,
        target: PeerId,
        protocol_name: Cow<'static, [u8]>,
        message: impl Into<Vec<u8>>,
    ) {
        let msg = Message::Consensus(ConsensusMessage {
            data: message.into(),
        })
        .encode();

        self.behaviour.write_notification(
            &target,
            protocol_name,
            msg.expect("should succ"),
            vec![],
        );
    }

    pub fn register_notifications_protocol(
        &mut self,
        protocol_name: impl Into<Cow<'static, [u8]>>,
    ) -> Vec<event::Event> {
        let protocol_name = protocol_name.into();
        self.behaviour
            .register_notif_protocol(protocol_name.clone(), Vec::new());

        info!(
            "register protocol {:?} successful",
            str::from_utf8(&protocol_name)
        );
        self.context_data
            .peers
            .iter()
            .map(|(peer_id, _peer)| event::Event::NotificationStreamOpened {
                remote: peer_id.clone(),
                info: Box::new(self.chain_info.self_info.clone()),
            })
            .collect()
    }

    pub fn update_self_info(&mut self, self_info: PeerInfo) {
        self.chain_info.self_info = self_info;
    }
}

fn send_message(
    behaviour: &mut GenericProto,
    who: &PeerId,
    message: Message,
) -> anyhow::Result<()> {
    let encoded = message.encode()?;
    behaviour.send_packet(who, encoded);
    Ok(())
}
