pub mod event;
pub mod generic_proto;
pub mod message;

use crate::protocol::generic_proto::{GenericProto, GenericProtoOut, NotificationsSink};
// use crate::protocol::message::generic::Status;
use crate::business_layer_handle::BusinessLayerHandle;
use crate::utils::interval;
use crate::{errors, DiscoveryNetBehaviour, Multiaddr};
use bytes::Bytes;
use futures::prelude::*;
use libp2p::core::connection::ConnectionId;
use libp2p::swarm::behaviour::FromSwarm;
use libp2p::swarm::{ConnectionHandler, IntoConnectionHandler};
use libp2p::swarm::{NetworkBehaviour, NetworkBehaviourAction, PollParameters};
use libp2p::PeerId;
use log::Level;
use sc_peerset::{peersstate::PeersState, SetId};
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
    /// Failed to encode message.
    pub const FAILED_TO_ENCODE: Rep = Rep::new_fatal("failed to encode message into Vec<u8>");
}

#[derive(Debug)]
pub enum CustomMessageOutcome {
    /// Notification protocols have been opened with a remote.
    NotificationStreamOpened {
        remote: PeerId,
        protocol: Cow<'static, str>,
        notifications_sink: NotificationsSink,
        generic_data: Vec<u8>,
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
    /// The data in peer is generic.
    /// As a network node, it does not need to know what the above layers do.  
    #[allow(unused)]
    info: Vec<u8>,
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

pub struct Protocol<T: 'static + BusinessLayerHandle + Send> {
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
    /// for interacting with the business layer above the network-p2p module
    business_layer_handle: T,
}

impl<T: BusinessLayerHandle + Send> NetworkBehaviour for Protocol<T> {
    type ConnectionHandler = <GenericProto as NetworkBehaviour>::ConnectionHandler;
    type OutEvent = CustomMessageOutcome;

    fn new_handler(&mut self) -> Self::ConnectionHandler {
        self.behaviour.new_handler()
    }

    fn addresses_of_peer(&mut self, peer_id: &PeerId) -> Vec<Multiaddr> {
        self.behaviour.addresses_of_peer(peer_id)
    }
    fn on_swarm_event(&mut self, event: FromSwarm<Self::ConnectionHandler>) {
        self.behaviour.on_swarm_event(event);
    }

    fn on_connection_handler_event(
        &mut self,
        peer_id: PeerId,
        connection_id: ConnectionId,
        event: <<Self::ConnectionHandler as IntoConnectionHandler>::Handler as
        ConnectionHandler>::OutEvent,
    ) {
        self.behaviour
            .on_connection_handler_event(peer_id, connection_id, event);
    }

    fn poll(
        &mut self,
        cx: &mut std::task::Context,
        params: &mut impl PollParameters,
    ) -> Poll<NetworkBehaviourAction<Self::OutEvent, Self::ConnectionHandler>> {
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
            } => {
                let result = self.business_layer_handle.handshake(
                    peer_id,
                    set_id,
                    self.notif_protocols[usize::from(set_id)].clone(),
                    received_handshake.clone(),
                    notifications_sink,
                );
                match result {
                    Ok(custom_message) => {
                        debug!(target: "network-p2p", "Connected {}", peer_id);
                        let peer = Peer {
                            info: received_handshake,
                        };
                        self.context_data.peers.insert(peer_id, peer);
                        debug!(target: "network-p2p", "Connected {}, Set id {:?}", peer_id, set_id);
                        custom_message
                    }
                    Err(err) => {
                        error!("business layer handle returned a failure: {:?}", err);
                        if err == rep::BAD_MESSAGE {
                            self.bad_handshake_substreams.insert((peer_id, set_id));
                            self.peerset_handle.report_peer(peer_id, err);
                            self.behaviour
                                .disconnect_peer(&peer_id, HARD_CORE_PROTOCOL_ID);
                        } else if err == rep::GENESIS_MISMATCH {
                            if self.boot_node_ids.contains(&peer_id) {
                                error!(
                                    target: "network-p2p",
                                    "Bootnode with peer id `{}` is on a different chain",
                                    peer_id,
                                );
                            } else {
                                log!(
                                    target: "network-p2p",
                                    if self.important_peers.contains(&peer_id) { Level::Warn } else { Level::Debug },
                                    "Peer with id `{}` is on different chain",
                                    peer_id
                                );
                            }
                            self.peerset_handle.report_peer(peer_id, err);
                        } else if err == rep::BAD_PROTOCOL {
                            log!(
                                target: "network-p2p",
                                if self.important_peers.contains(&peer_id) { Level::Warn } else { Level::Debug },
                                "Peer {:?} using unsupported protocol version", peer_id
                            );
                            self.peerset_handle.report_peer(peer_id, err);
                        }
                        CustomMessageOutcome::None
                    }
                }
            }
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

impl<T: BusinessLayerHandle + Send> DiscoveryNetBehaviour for Protocol<T> {
    fn add_discovered_nodes(&mut self, peer_ids: impl Iterator<Item = PeerId>) {
        for peer_id in peer_ids {
            for (set_id, _) in self.notif_protocols.iter().enumerate() {
                self.peerset_handle
                    .add_to_peers_set(SetId::from(set_id), peer_id);
            }
        }
    }
}

impl<T: 'static + BusinessLayerHandle + Send> Protocol<T> {
    /// Create a new instance.
    pub fn new(
        peerset_config: sc_peerset::PeersetConfig,
        mut business_layer_handle: T,
        boot_node_ids: Arc<HashSet<PeerId>>,
        notif_protocols: Vec<Cow<'static, str>>,
        rpc_protocols: Vec<Cow<'static, str>>,
    ) -> errors::Result<(Protocol<T>, sc_peerset::PeersetHandle)> {
        let mut important_peers = HashSet::new();
        important_peers.extend(boot_node_ids.iter());
        for peer_set in peerset_config.sets.iter() {
            for reserved in &peer_set.reserved_nodes {
                important_peers.insert(*reserved);
            }
        }

        let (peerset, peerset_handle) = sc_peerset::Peerset::from_config(peerset_config);
        let behaviour = {
            let handshake_message = business_layer_handle
                .build_handshake_msg(notif_protocols.to_vec(), rpc_protocols.to_vec())
                .expect("Status encode should success.");
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
            business_layer_handle,
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

    pub fn update_status(&mut self, status: &[u8]) {
        let _ = self.business_layer_handle.update_status(status);
        self.update_handshake()
            .expect("update status should success.");
    }

    fn update_handshake(&mut self) -> anyhow::Result<()> {
        let handshake_msg = self
            .business_layer_handle
            .build_handshake_msg(self.notif_protocols.to_vec(), self.rpc_protocols.to_vec())
            .expect("Status encode should success.");
        for (set_id, _) in self.notif_protocols.iter().enumerate() {
            self.behaviour
                .set_notif_protocol_handshake(SetId::from(set_id), handshake_msg.clone());
        }
        Ok(())
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
    /// Notify the protocol that we have learned about the existence of nodes on the default set.
    ///
    /// Can be called multiple times with the same `PeerId`s.
    pub fn add_default_set_discovered_nodes(&mut self, peer_ids: impl Iterator<Item = PeerId>) {
        for peer_id in peer_ids {
            self.peerset_handle
                .add_to_peers_set(HARD_CORE_PROTOCOL_ID, peer_id);
        }
    }
}

impl<T: BusinessLayerHandle + Send> Drop for Protocol<T> {
    fn drop(&mut self) {
        debug!(target: "sync", "Network stats:\n{}", self.format_stats());
    }
}
