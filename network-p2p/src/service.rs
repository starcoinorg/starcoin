// Copyright 2017-2020 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

//! Main entry point of the sc-network crate.
//!
//! There are two main structs in this module: [`NetworkWorker`] and [`NetworkService`].
//! The [`NetworkWorker`] *is* the network and implements the `Future` trait. It must be polled in
//! order fo the network to advance.
//! The [`NetworkService`] is merely a shared version of the [`NetworkWorker`]. You can obtain an
//! `Arc<NetworkService>` by calling [`NetworkWorker::service`].
//!
//! The methods of the [`NetworkService`] are implemented by sending a message over a channel,
//! which is then processed by [`NetworkWorker::poll`].

use std::pin::Pin;
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc,
};
use std::task::Poll;
use std::{borrow::Cow, collections::HashSet, io};

use futures::{
    channel::{mpsc, oneshot},
    prelude::*,
};
use libp2p::core::{
    connection::{ConnectionError, PendingConnectionError},
    ConnectedPoint,
};
use libp2p::swarm::{
    protocols_handler::NodeHandlerWrapperError, NetworkBehaviour, SwarmBuilder, SwarmEvent,
};
use libp2p::{kad::record, PeerId};
use log::{error, info, trace, warn};
use parking_lot::Mutex;
use sc_peerset::{PeersetHandle, ReputationChange};
use starcoin_types::peer_info::PeerInfo;

use crate::config::{Params, TransportConfig};
use crate::discovery::DiscoveryConfig;
use crate::errors::Error;
use crate::metrics::Metrics;
use crate::network_state::{
    NetworkState, NotConnectedPeer as NetworkStateNotConnectedPeer, Peer as NetworkStatePeer,
};
use crate::protocol::event::Event;
use crate::protocol::generic_proto::{NotificationsSink, Ready};
use crate::protocol::{ChainInfo, Protocol};
use crate::{
    behaviour::{Behaviour, BehaviourOut},
    errors, out_events, DhtEvent,
};
use crate::{
    config::{parse_addr, parse_str_addr, NonReservedPeerMode},
    transport,
};
use crate::{Multiaddr, PROTOCOL_NAME};
use starcoin_metrics::{Histogram, HistogramVec};
use std::collections::HashMap;
use std::num::NonZeroUsize;

/// Minimum Requirements for a Hash within Networking
pub trait ExHashT: std::hash::Hash + Eq + std::fmt::Debug + Clone + Send + Sync + 'static {}

impl<T> ExHashT for T where T: std::hash::Hash + Eq + std::fmt::Debug + Clone + Send + Sync + 'static
{}

/// A cloneable handle for reporting cost/benefits of peers.
#[derive(Clone)]
pub struct ReportHandle {
    inner: PeersetHandle, // wraps it so we don't have to worry about breaking API.
}

impl From<PeersetHandle> for ReportHandle {
    fn from(peerset_handle: PeersetHandle) -> Self {
        ReportHandle {
            inner: peerset_handle,
        }
    }
}

/// Substrate network service. Handles network IO and manages connectivity.
pub struct NetworkService {
    /// Number of peers we're connected to.
    num_connected: Arc<AtomicUsize>,
    /// The local external addresses.
    external_addresses: Arc<Mutex<Vec<Multiaddr>>>,
    /// Are we actively catching up with the chain?
    is_major_syncing: Arc<AtomicBool>,
    /// Local copy of the `PeerId` of the local node.
    local_peer_id: PeerId,
    /// Bandwidth logging system. Can be queried to know the average bandwidth consumed.
    bandwidth: Arc<transport::BandwidthSinks>,
    /// Peerset manager (PSM); manages the reputation of nodes and indicates the network which
    /// nodes it should be connected to or not.
    peerset: PeersetHandle,
    /// For each peer and protocol combination, an object that allows sending notifications to
    /// that peer. Updated by the [`NetworkWorker`].
    peers_notifications_sinks: Arc<Mutex<HashMap<(PeerId, Cow<'static, str>), NotificationsSink>>>,
    /// Channel that sends messages to the actual worker.
    to_worker: mpsc::UnboundedSender<ServiceToWorkerMsg>,
    /// Field extracted from the [`Metrics`] struct and necessary to report the
    /// notifications-related metrics.
    notifications_sizes_metric: Option<HistogramVec>,
}

impl NetworkWorker {
    /// Creates the network service.
    ///
    /// Returns a `NetworkWorker` that implements `Future` and must be regularly polled in order
    /// for the network processing to advance. From it, you can extract a `NetworkService` using
    /// `worker.service()`. The `NetworkService` can be shared through the codebase.
    pub fn new(params: Params) -> errors::Result<NetworkWorker> {
        // Ensure the listen addresses are consistent with the transport.
        ensure_addresses_consistent_with_transport(
            params.network_config.listen_addresses.iter(),
            &params.network_config.transport,
        )?;
        ensure_addresses_consistent_with_transport(
            params
                .network_config
                .boot_nodes
                .iter()
                .map(|x| &x.multiaddr),
            &params.network_config.transport,
        )?;
        ensure_addresses_consistent_with_transport(
            params
                .network_config
                .reserved_nodes
                .iter()
                .map(|x| &x.multiaddr),
            &params.network_config.transport,
        )?;
        ensure_addresses_consistent_with_transport(
            params.network_config.public_addresses.iter(),
            &params.network_config.transport,
        )?;

        let (to_worker, from_worker) = mpsc::unbounded();

        // List of multiaddresses that we know in the network.
        let mut known_addresses = Vec::new();
        let mut bootnodes = Vec::new();
        let mut boot_node_ids = HashSet::new();

        // Process the bootnodes.
        for bootnode in params.network_config.boot_nodes.iter() {
            bootnodes.push(bootnode.peer_id.clone());
            boot_node_ids.insert(bootnode.peer_id.clone());
            known_addresses.push((bootnode.peer_id.clone(), bootnode.multiaddr.clone()));
        }

        let boot_node_ids = Arc::new(boot_node_ids);

        // Check for duplicate bootnodes.
        known_addresses.iter().try_for_each(|(peer_id, addr)| {
            if let Some(other) = known_addresses
                .iter()
                .find(|o| o.1 == *addr && o.0 != *peer_id)
            {
                Err(Error::DuplicateBootnode {
                    address: addr.clone(),
                    first_id: peer_id.clone(),
                    second_id: other.0.clone(),
                })
            } else {
                Ok(())
            }
        })?;

        let priority_groups = {
            let mut reserved_nodes = HashSet::new();
            for reserved in params.network_config.reserved_nodes.iter() {
                reserved_nodes.insert(reserved.peer_id.clone());
                known_addresses.push((reserved.peer_id.clone(), reserved.multiaddr.clone()));
            }

            vec![("reserved".to_owned(), reserved_nodes)]
        };

        let peerset_config = sc_peerset::PeersetConfig {
            in_peers: params.network_config.in_peers,
            out_peers: params.network_config.out_peers,
            bootnodes,
            reserved_only: params.network_config.non_reserved_mode == NonReservedPeerMode::Deny,
            priority_groups,
        };

        // Private and public keys configuration.
        let local_identity = params.network_config.node_key.clone().into_keypair()?;
        let local_public = local_identity.public();
        let local_peer_id = local_public.clone().into_peer_id();
        info!(target: "sub-libp2p", "Local node identity is: {}", local_peer_id.to_base58());

        let num_connected = Arc::new(AtomicUsize::new(0));
        let is_major_syncing = Arc::new(AtomicBool::new(false));
        let chain_info = ChainInfo {
            genesis_hash: params.network_config.genesis_hash,
            self_info: params.network_config.self_info,
        };
        let mut notif_protocols = params.network_config.protocols.clone();
        notif_protocols.push(PROTOCOL_NAME.into());

        let (protocol, peerset_handle) = Protocol::new(
            peerset_config,
            local_peer_id.clone(),
            params.protocol_id.clone(),
            chain_info,
            boot_node_ids.clone(),
            notif_protocols,
        )?;

        // Build the swarm.
        let (mut swarm, bandwidth): (Swarm, _) = {
            let user_agent = format!(
                "{} ({})",
                params.network_config.client_version, params.network_config.node_name
            );

            let discovery_config = {
                let mut config = DiscoveryConfig::new(local_public.clone());
                config.with_user_defined(known_addresses);
                config.discovery_limit(u64::from(params.network_config.out_peers) + 15);
                config.add_protocol(params.protocol_id.clone());
                config.allow_non_globals_in_dht(false);

                match params.network_config.transport {
                    TransportConfig::MemoryOnly => {
                        config.with_mdns(false);
                        config.allow_private_ipv4(false);
                    }
                    TransportConfig::Normal {
                        enable_mdns,
                        allow_private_ipv4,
                        ..
                    } => {
                        config.with_mdns(enable_mdns);
                        config.allow_private_ipv4(allow_private_ipv4);
                    }
                }

                config
            };

            // protocol.register_notifications_protocol(PROTOCOL_NAME.into());
            // for protocol_name in params.network_config.protocols {
            //     protocol.register_notifications_protocol(protocol_name);
            // }

            let mut behaviour =
                Behaviour::new(protocol, user_agent, local_public, discovery_config);

            behaviour.register_notifications_protocol(Cow::Borrowed(PROTOCOL_NAME));
            for protocol_name in &params.network_config.protocols {
                behaviour.register_notifications_protocol(protocol_name.clone());
            }

            let (transport, bandwidth) = {
                let (config_mem, config_wasm) = match params.network_config.transport {
                    TransportConfig::MemoryOnly => (true, None),
                    TransportConfig::Normal {
                        wasm_external_transport,
                        ..
                    } => (false, wasm_external_transport),
                };
                transport::build_transport(local_identity, config_mem, config_wasm)
            };
            let builder = SwarmBuilder::new(transport, behaviour, local_peer_id.clone())
                .peer_connection_limit(crate::MAX_CONNECTIONS_PER_PEER)
                .notify_handler_buffer_size(NonZeroUsize::new(32).expect("32 != 0; qed"))
                .connection_event_buffer_size(1024);

            (builder.build(), bandwidth)
        };

        // Listen on multiaddresses.
        for addr in &params.network_config.listen_addresses {
            if let Err(err) = Swarm::listen_on(&mut swarm, addr.clone()) {
                warn!(target: "sub-libp2p", "Can't listen on {} because: {:?}", addr, err)
            }
        }

        // Add external addresses.
        for addr in &params.network_config.public_addresses {
            Swarm::add_external_address(&mut swarm, addr.clone());
        }

        let external_addresses = Arc::new(Mutex::new(Vec::new()));
        let peers_notifications_sinks = Arc::new(Mutex::new(HashMap::new()));

        let metrics = params
            .metrics_registry
            .as_ref()
            .and_then(|registry| Metrics::register(&registry).ok());
        let service = Arc::new(NetworkService {
            bandwidth,
            external_addresses,
            num_connected,
            is_major_syncing,
            peerset: peerset_handle,
            local_peer_id,
            peers_notifications_sinks: peers_notifications_sinks.clone(),
            to_worker,
            notifications_sizes_metric: metrics
                .as_ref()
                .map(|metrics| metrics.notifications_sizes.clone()),
        });

        Ok(NetworkWorker {
            network_service: swarm,
            service,
            from_worker,
            event_streams: out_events::OutChannels::new(params.metrics_registry.as_ref())?,
            metrics,
            boot_node_ids,
            peers_notifications_sinks,
        })
    }

    /// Returns the total number of bytes received so far.
    pub fn total_bytes_inbound(&self) -> u64 {
        self.service.bandwidth.total_inbound()
    }

    /// Returns the total number of bytes sent so far.
    pub fn total_bytes_outbound(&self) -> u64 {
        self.service.bandwidth.total_outbound()
    }

    /// Returns the number of peers we're connected to.
    pub fn num_connected_peers(&self) -> usize {
        self.network_service.user_protocol().num_connected_peers()
    }

    // /// Returns the number of peers we're connected to and that are being queried.
    // pub fn num_active_peers(&self) -> usize {
    //     self.network_service.user_protocol().num_active_peers()
    // }

    /// Adds an address for a node.
    pub fn add_known_address(&mut self, peer_id: PeerId, addr: Multiaddr) {
        self.network_service.add_known_address(peer_id, addr);
    }

    /// Return a `NetworkService` that can be shared through the code base and can be used to
    /// manipulate the worker.
    pub fn service(&self) -> &Arc<NetworkService> {
        &self.service
    }

    /// Get network state.
    ///
    /// **Note**: Use this only for debugging. This API is unstable. There are warnings literally
    /// everywhere about this. Please don't use this function to retrieve actual information.
    pub fn network_state(&mut self) -> NetworkState {
        let swarm = &mut self.network_service;
        let open = swarm
            .user_protocol()
            .open_peers()
            .cloned()
            .collect::<Vec<_>>();

        let connected_peers = {
            let swarm = &mut *swarm;
            open.iter().filter_map(move |peer_id| {
                let known_addresses = NetworkBehaviour::addresses_of_peer(&mut **swarm, peer_id)
                    .into_iter().collect();

                let endpoint = if let Some(e) = swarm.node(peer_id).map(|i| i.endpoint()) {
                    e.clone().into()
                } else {
                    error!(target: "sub-libp2p", "Found state inconsistency between custom protocol \
						and debug information about {:?}", peer_id);
                    return None
                };

                Some((peer_id.to_base58(), NetworkStatePeer {
                    endpoint,
                    version_string: swarm.node(peer_id)
                        .and_then(|i| i.client_version().map(|s| s.to_owned())),
                    latest_ping_time: swarm.node(peer_id).and_then(|i| i.latest_ping()),
                    enabled: swarm.user_protocol().is_enabled(&peer_id),
                    open: swarm.user_protocol().is_open(&peer_id),
                    known_addresses,
                }))
            }).collect()
        };

        let not_connected_peers = {
            let swarm = &mut *swarm;
            swarm
                .known_peers()
                .into_iter()
                .filter(|p| open.iter().all(|n| n != p))
                .map(move |peer_id| {
                    (
                        peer_id.to_base58(),
                        NetworkStateNotConnectedPeer {
                            version_string: swarm
                                .node(&peer_id)
                                .and_then(|i| i.client_version().map(|s| s.to_owned())),
                            latest_ping_time: swarm.node(&peer_id).and_then(|i| i.latest_ping()),
                            known_addresses: NetworkBehaviour::addresses_of_peer(
                                &mut **swarm,
                                &peer_id,
                            )
                            .into_iter()
                            .collect(),
                        },
                    )
                })
                .collect()
        };

        NetworkState {
            peer_id: Swarm::local_peer_id(&swarm).to_base58(),
            listened_addresses: Swarm::listeners(&swarm).cloned().collect(),
            external_addresses: Swarm::external_addresses(&swarm).cloned().collect(),
            connected_peers,
            not_connected_peers,
            peerset: swarm.user_protocol_mut().peerset_debug_info(),
        }
    }

    /// Removes a `PeerId` from the list of reserved peers.
    pub fn remove_reserved_peer(&self, peer: PeerId) {
        self.service.remove_reserved_peer(peer);
    }

    /// Adds a `PeerId` and its address as reserved. The string should encode the address
    /// and peer ID of the remote node.
    pub fn add_reserved_peer(&self, peer: String) -> Result<(), String> {
        self.service.add_reserved_peer(peer)
    }

    /// Returns the list of all the peers we are connected to.
    pub fn connected_peers(&mut self) -> HashSet<PeerId> {
        self.network_service.known_peers()
    }

    pub fn is_open(&self, peer_id: &PeerId) -> bool {
        self.network_service.is_open(peer_id)
    }
}

impl NetworkService {
    /// Writes a message on an open notifications channel. Has no effect if the notifications
    /// channel with this protocol name is closed.
    ///
    /// > **Note**: The reason why this is a no-op in the situation where we have no channel is
    /// >        that we don't guarantee message delivery anyway. Networking issues can cause
    /// >        connections to drop at any time, and higher-level logic shouldn't differentiate
    /// >        between the remote voluntarily closing a substream or a network error
    /// >        preventing the message from being delivered.
    ///
    /// The protocol must have been registered with `register_notifications_protocol`.
    ///
    pub fn write_notification(
        &self,
        target: PeerId,
        protocol_name: Cow<'static, str>,
        message: Vec<u8>,
    ) {
        // We clone the `NotificationsSink` in order to be able to unlock the network-wide
        // `peers_notifications_sinks` mutex as soon as possible.
        let sink = {
            let peers_notifications_sinks = self.peers_notifications_sinks.lock();
            if let Some(sink) = peers_notifications_sinks.get(&(target, protocol_name.clone())) {
                sink.clone()
            } else {
                // Notification silently discarded, as documented.
                return;
            }
        };

        // Used later for the metrics report.
        //let message_len = message.len();

        sink.send_sync_notification(protocol_name, message);

        // if let Some(notifications_sizes_metric) = self.notifications_sizes_metric.as_ref() {
        //     notifications_sizes_metric
        //         .with_label_values(&["out", &maybe_utf8_bytes_to_string(&engine_id)])
        //         .observe(message_len as f64);
        // }
    }

    /// Obtains a [`NotificationSender`] for a connected peer, if it exists.
    ///
    /// A `NotificationSender` is scoped to a particular connection to the peer that holds
    /// a receiver. With a `NotificationSender` at hand, sending a notification is done in two steps:
    ///
    /// 1.  [`NotificationSender::ready`] is used to wait for the sender to become ready
    /// for another notification, yielding a [`NotificationSenderReady`] token.
    /// 2.  [`NotificationSenderReady::send`] enqueues the notification for sending. This operation
    /// can only fail if the underlying notification substream or connection has suddenly closed.
    ///
    /// An error is returned by [`NotificationSenderReady::send`] if there exists no open
    /// notifications substream with that combination of peer and protocol, or if the remote
    /// has asked to close the notifications substream. If that happens, it is guaranteed that an
    /// [`Event::NotificationStreamClosed`] has been generated on the stream returned by
    /// [`NetworkService::event_stream`].
    ///
    /// If the remote requests to close the notifications substream, all notifications successfully
    /// enqueued using [`NotificationSenderReady::send`] will finish being sent out before the
    /// substream actually gets closed, but attempting to enqueue more notifications will now
    /// return an error. It is however possible for the entire connection to be abruptly closed,
    /// in which case enqueued notifications will be lost.
    ///
    /// The protocol must have been registered with `register_notifications_protocol` or
    /// [`NetworkConfiguration::notifications_protocols`](crate::config::NetworkConfiguration::notifications_protocols).
    ///
    /// # Usage
    ///
    /// This method returns a struct that allows waiting until there is space available in the
    /// buffer of messages towards the given peer. If the peer processes notifications at a slower
    /// rate than we send them, this buffer will quickly fill up.
    ///
    /// As such, you should never do something like this:
    ///
    /// ```ignore
    /// // Do NOT do this
    /// for peer in peers {
    ///     if let Ok(n) = network.notification_sender(peer, ...) {
    ///         if let Ok(s) = n.ready().await {
    ///             let _ = s.send(...);
    ///        }
    ///     }
    /// }
    /// ```
    ///
    /// Doing so would slow down all peers to the rate of the slowest one. A malicious or
    /// malfunctioning peer could intentionally process notifications at a very slow rate.
    ///
    /// Instead, you are encouraged to maintain your own buffer of notifications on top of the one
    /// maintained by `sc-network`, and use `notification_sender` to progressively send out
    /// elements from your buffer. If this additional buffer is full (which will happen at some
    /// point if the peer is too slow to process notifications), appropriate measures can be taken,
    /// such as removing non-critical notifications from the buffer or disconnecting the peer
    /// using [`NetworkService::disconnect_peer`].
    ///
    ///
    /// Notifications              Per-peer buffer
    ///   broadcast    +------->   of notifications   +-->  `notification_sender`  +-->  Internet
    ///                    ^       (not covered by
    ///                    |         sc-network)
    ///                    +
    ///      Notifications should be dropped
    ///             if buffer is full
    ///
    ///
    /// See also the [`gossip`](crate::gossip) module for a higher-level way to send
    /// notifications.
    ///
    pub fn notification_sender(
        &self,
        target: PeerId,
        protocol_name: Cow<'static, str>,
    ) -> Result<NotificationSender, NotificationSenderError> {
        // We clone the `NotificationsSink` in order to be able to unlock the network-wide
        // `peers_notifications_sinks` mutex as soon as possible.
        let sink = {
            let peers_notifications_sinks = self.peers_notifications_sinks.lock();
            if let Some(sink) = peers_notifications_sinks.get(&(target, protocol_name.clone())) {
                sink.clone()
            } else {
                return Err(NotificationSenderError::Closed);
            }
        };

        Ok(NotificationSender {
            sink,
            protocol_name: protocol_name.clone(),
            notification_size_metric: self
                .notifications_sizes_metric
                .as_ref()
                .map(|histogram| histogram.with_label_values(&["out", &protocol_name])),
        })
    }

    pub async fn broadcast_message(&self, protocol_name: Cow<'static, str>, message: Vec<u8>) {
        debug!("start send broadcast message");

        let peers = self.connected_peers().await;
        for peer_id in peers {
            self.write_notification(peer_id, protocol_name.clone(), message.clone());
        }
        debug!("finish send broadcast message");
    }

    pub async fn is_connected(&self, address: PeerId) -> bool {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .to_worker
            .unbounded_send(ServiceToWorkerMsg::IsConnected(address, tx));
        match rx.await {
            Ok(t) => t,
            Err(e) => {
                warn!("sth wrong {}", e);
                false
            }
        }
    }

    pub async fn connected_peers(&self) -> HashSet<PeerId> {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .to_worker
            .unbounded_send(ServiceToWorkerMsg::ConnectedPeers(tx));
        match rx.await {
            Ok(t) => t,
            Err(e) => {
                debug!("sth wrong {}", e);
                HashSet::new()
            }
        }
    }

    pub async fn get_address(&self, peer_id: PeerId) -> Vec<Multiaddr> {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .to_worker
            .unbounded_send(ServiceToWorkerMsg::AddressByPeerID(peer_id, tx));
        match rx.await {
            Ok(t) => t,
            Err(e) => {
                debug!("sth wrong {}", e);
                Vec::new()
            }
        }
    }

    pub fn update_self_info(&self, info: PeerInfo) {
        let _ = self
            .to_worker
            .unbounded_send(ServiceToWorkerMsg::SelfInfo(Box::new(info)));
    }

    /// Returns a stream containing the events that happen on the network.
    ///
    /// If this method is called multiple times, the events are duplicated.
    ///
    /// The stream never ends (unless the `NetworkWorker` gets shut down).
    ///
    /// The name passed is used to identify the channel in the Prometheus metrics. Note that the
    /// parameter is a `&'static str`, and not a `String`, in order to avoid accidentally having
    /// an unbounded set of Prometheus metrics, which would be quite bad in terms of memory
    pub fn event_stream(&self, name: &'static str) -> impl Stream<Item = Event> {
        let (tx, rx) = out_events::channel(name);
        let _ = self
            .to_worker
            .unbounded_send(ServiceToWorkerMsg::EventStream(tx));
        rx
    }

    /// Registers a new notifications protocol.
    ///
    /// After that, you can call `write_notifications`.
    ///
    /// Please call `event_stream` before registering a protocol, otherwise you may miss events
    /// about the protocol that you have registered.
    ///
    /// You are very strongly encouraged to call this method very early on. Any connection open
    /// will retain the protocols that were registered then, and not any new one.
    pub fn register_notifications_protocol(&self, protocol_name: impl Into<Cow<'static, str>>) {
        let _ = self
            .to_worker
            .unbounded_send(ServiceToWorkerMsg::RegisterNotifProtocol {
                protocol_name: protocol_name.into(),
            });
    }

    /// Report a given peer as either beneficial (+) or costly (-) according to the
    /// given scalar.
    pub fn report_peer(&self, who: PeerId, cost_benefit: ReputationChange) {
        self.peerset.report_peer(who, cost_benefit);
    }

    /// Disconnect from a node as soon as possible.
    ///
    /// This triggers the same effects as if the connection had closed itself spontaneously.
    pub fn disconnect_peer(&self, who: PeerId) {
        let _ = self
            .to_worker
            .unbounded_send(ServiceToWorkerMsg::DisconnectPeer(who));
    }

    /// Are we in the process of downloading the chain?
    pub fn is_major_syncing(&self) -> bool {
        self.is_major_syncing.load(Ordering::Relaxed)
    }

    /// Start getting a value from the DHT.
    ///
    /// This will generate either a `ValueFound` or a `ValueNotFound` event and pass it as an
    /// item on the [`NetworkWorker`] stream.
    pub fn get_value(&self, key: &record::Key) {
        let _ = self
            .to_worker
            .unbounded_send(ServiceToWorkerMsg::GetValue(key.clone()));
    }

    /// Start putting a value in the DHT.
    ///
    /// This will generate either a `ValuePut` or a `ValuePutFailed` event and pass it as an
    /// item on the [`NetworkWorker`] stream.
    pub fn put_value(&self, key: record::Key, value: Vec<u8>) {
        let _ = self
            .to_worker
            .unbounded_send(ServiceToWorkerMsg::PutValue(key, value));
    }

    /// Connect to unreserved peers and allow unreserved peers to connect.
    pub fn accept_unreserved_peers(&self) {
        self.peerset.set_reserved_only(false);
    }

    /// Disconnect from unreserved peers and deny new unreserved peers to connect.
    pub fn deny_unreserved_peers(&self) {
        self.peerset.set_reserved_only(true);
    }

    /// Removes a `PeerId` from the list of reserved peers.
    pub fn remove_reserved_peer(&self, peer: PeerId) {
        self.peerset.remove_reserved_peer(peer);
    }

    /// Adds a `PeerId` and its address as reserved. The string should encode the address
    /// and peer ID of the remote node.
    pub fn add_reserved_peer(&self, peer: String) -> Result<(), String> {
        let (peer_id, addr) = parse_str_addr(&peer).map_err(|e| format!("{:?}", e))?;
        self.peerset.add_reserved_peer(peer_id.clone());
        let _ = self
            .to_worker
            .unbounded_send(ServiceToWorkerMsg::AddKnownAddress(peer_id, addr));
        Ok(())
    }

    /// Modify a peerset priority group.
    pub fn set_priority_group(
        &self,
        group_id: String,
        peers: HashSet<Multiaddr>,
    ) -> Result<(), String> {
        let peers = peers
            .into_iter()
            .map(|p| parse_addr(p).map_err(|e| format!("{:?}", e)))
            .collect::<Result<Vec<(PeerId, Multiaddr)>, String>>()?;

        let peer_ids = peers
            .iter()
            .map(|(peer_id, _addr)| peer_id.clone())
            .collect();
        self.peerset.set_priority_group(group_id, peer_ids);

        for (peer_id, addr) in peers.into_iter() {
            let _ = self
                .to_worker
                .unbounded_send(ServiceToWorkerMsg::AddKnownAddress(peer_id, addr));
        }

        Ok(())
    }

    /// Returns the number of peers we're connected to.
    pub fn num_connected(&self) -> usize {
        self.num_connected.load(Ordering::Relaxed)
    }

    pub fn peer_id(&self) -> &PeerId {
        &self.local_peer_id
    }

    pub async fn exist_notif_proto(&self, protocol_name: Cow<'static, str>) -> bool {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .to_worker
            .unbounded_send(ServiceToWorkerMsg::ExistNotifProtocol { protocol_name, tx });
        match rx.await {
            Ok(t) => t,
            Err(e) => {
                warn!("sth wrong {}", e);
                false
            }
        }
    }
}

/// Trait for providing information about the local network state
pub trait NetworkStateInfo {
    /// Returns the local external addresses.
    fn external_addresses(&self) -> Vec<Multiaddr>;

    /// Returns the local Peer ID.
    fn local_peer_id(&self) -> PeerId;
}

impl NetworkStateInfo for NetworkService {
    /// Returns the local external addresses.
    fn external_addresses(&self) -> Vec<Multiaddr> {
        self.external_addresses.lock().clone()
    }

    /// Returns the local Peer ID.
    fn local_peer_id(&self) -> PeerId {
        self.local_peer_id.clone()
    }
}

/// A `NotificationSender` allows for sending notifications to a peer with a chosen protocol.
#[must_use]
pub struct NotificationSender {
    sink: NotificationsSink,

    /// Name of the protocol on the wire.
    protocol_name: Cow<'static, str>,

    /// Field extracted from the [`Metrics`] struct and necessary to report the
    /// notifications-related metrics.
    notification_size_metric: Option<Histogram>,
}

impl NotificationSender {
    /// Returns a future that resolves when the `NotificationSender` is ready to send a notification.
    pub async fn ready(&self) -> Result<NotificationSenderReady<'_>, NotificationSenderError> {
        Ok(NotificationSenderReady {
            ready: match self
                .sink
                .reserve_notification(self.protocol_name.clone())
                .await
            {
                Ok(r) => r,
                Err(()) => return Err(NotificationSenderError::Closed),
            },
            notification_size_metric: self.notification_size_metric.clone(),
        })
    }
}

/// Reserved slot in the notifications buffer, ready to accept data.
#[must_use]
pub struct NotificationSenderReady<'a> {
    ready: Ready<'a>,

    /// Field extracted from the [`Metrics`] struct and necessary to report the
    /// notifications-related metrics.
    notification_size_metric: Option<Histogram>,
}

impl<'a> NotificationSenderReady<'a> {
    /// Consumes this slots reservation and actually queues the notification.
    pub fn send(self, notification: impl Into<Vec<u8>>) -> Result<(), NotificationSenderError> {
        let notification = notification.into();

        if let Some(notification_size_metric) = &self.notification_size_metric {
            notification_size_metric.observe(notification.len() as f64);
        }

        self.ready
            .send(notification)
            .map_err(|()| NotificationSenderError::Closed)
    }
}

/// Error returned by [`NetworkService::send_notification`].
#[derive(Debug, derive_more::Display, derive_more::Error)]
pub enum NotificationSenderError {
    /// The notification receiver has been closed, usually because the underlying connection closed.
    ///
    /// Some of the notifications most recently sent may not have been received. However,
    /// the peer may still be connected and a new `NotificationSender` for the same
    /// protocol obtained from [`NetworkService::notification_sender`].
    Closed,
    /// Protocol name hasn't been registered.
    BadProtocol,
}

/// Messages sent from the `NetworkService` to the `NetworkWorker`.
///
/// Each entry corresponds to a method of `NetworkService`.
enum ServiceToWorkerMsg {
    GetValue(record::Key),
    PutValue(record::Key, Vec<u8>),
    AddKnownAddress(PeerId, Multiaddr),
    EventStream(out_events::Sender),
    RegisterNotifProtocol {
        protocol_name: Cow<'static, str>,
    },
    DisconnectPeer(PeerId),
    IsConnected(PeerId, oneshot::Sender<bool>),
    ConnectedPeers(oneshot::Sender<HashSet<PeerId>>),
    SelfInfo(Box<PeerInfo>),
    AddressByPeerID(PeerId, oneshot::Sender<Vec<Multiaddr>>),
    ExistNotifProtocol {
        protocol_name: Cow<'static, str>,
        tx: oneshot::Sender<bool>,
    },
}

/// Main network worker. Must be polled in order for the network to advance.
///
/// You are encouraged to poll this in a separate background thread or task.
#[must_use = "The NetworkWorker must be polled in order for the network to work"]
pub struct NetworkWorker {
    /// The network service that can be extracted and shared through the codebase.
    service: Arc<NetworkService>,
    /// The *actual* network.
    network_service: Swarm,
    /// Messages from the `NetworkService` and that must be processed.
    from_worker: mpsc::UnboundedReceiver<ServiceToWorkerMsg>,
    /// Senders for events that happen on the network.
    event_streams: out_events::OutChannels,
    /// Prometheus network metrics.
    metrics: Option<Metrics>,
    /// The `PeerId`'s of all boot nodes.
    boot_node_ids: Arc<HashSet<PeerId>>,
    /// For each peer and protocol combination, an object that allows sending notifications to
    /// that peer. Shared with the [`NetworkService`].
    peers_notifications_sinks: Arc<Mutex<HashMap<(PeerId, Cow<'static, str>), NotificationsSink>>>,
}

impl Future for NetworkWorker {
    type Output = Result<(), io::Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut std::task::Context) -> Poll<Self::Output> {
        let this = &mut *self;

        loop {
            // Process the next message coming from the `NetworkService`.
            let msg = match this.from_worker.poll_next_unpin(cx) {
                Poll::Ready(Some(msg)) => msg,
                Poll::Ready(None) => return Poll::Ready(Ok(())),
                Poll::Pending => break,
            };

            match msg {
                ServiceToWorkerMsg::GetValue(key) => this.network_service.get_value(&key),
                ServiceToWorkerMsg::PutValue(key, value) => {
                    this.network_service.put_value(key, value)
                }
                ServiceToWorkerMsg::AddKnownAddress(peer_id, addr) => {
                    this.network_service.add_known_address(peer_id, addr)
                }
                ServiceToWorkerMsg::EventStream(sender) => this.event_streams.push(sender),
                ServiceToWorkerMsg::RegisterNotifProtocol { protocol_name } => {
                    this.network_service
                        .register_notifications_protocol(protocol_name);
                }
                ServiceToWorkerMsg::DisconnectPeer(who) => this
                    .network_service
                    .user_protocol_mut()
                    .disconnect_peer(&who),
                ServiceToWorkerMsg::IsConnected(who, tx) => {
                    let _ = tx.send(this.is_open(&who));
                }
                ServiceToWorkerMsg::ConnectedPeers(tx) => {
                    let peers = this.connected_peers();
                    let mut result = HashSet::new();
                    for peer in peers {
                        result.insert(peer.clone());
                    }
                    let _ = tx.send(result);
                }
                ServiceToWorkerMsg::SelfInfo(info) => {
                    this.network_service
                        .user_protocol_mut()
                        .update_self_info(*info);
                }
                ServiceToWorkerMsg::AddressByPeerID(peer_id, tx) => {
                    let _ = tx.send(this.network_service.get_address(&peer_id));
                }
                ServiceToWorkerMsg::ExistNotifProtocol { protocol_name, tx } => {
                    let _ = tx.send(
                        this.network_service
                            .user_protocol()
                            .exist_notif_protocol(protocol_name),
                    );
                }
            }
        }

        // `num_iterations` serves the same purpose as in the previous loop.
        // See the previous loop for explanations.
        let mut num_iterations = 0;
        loop {
            num_iterations += 1;
            if num_iterations >= 1000 {
                cx.waker().wake_by_ref();
                break;
            }

            // Process the next action coming from the network.
            let next_event = this.network_service.next_event();
            futures::pin_mut!(next_event);
            let poll_value = next_event.poll_unpin(cx);

            match poll_value {
                Poll::Pending => break,
                Poll::Ready(SwarmEvent::Behaviour(BehaviourOut::RandomKademliaStarted(_))) => {}
                Poll::Ready(SwarmEvent::Behaviour(BehaviourOut::NotificationStreamOpened {
                    remote,
                    protocol,
                    notifications_sink,
                    info,
                })) => {
                    if let Some(metrics) = this.metrics.as_ref() {
                        metrics
                            .notifications_streams_opened_total
                            .with_label_values(&[&protocol])
                            .inc();
                    }
                    {
                        let mut peers_notifications_sinks = this.peers_notifications_sinks.lock();
                        peers_notifications_sinks
                            .insert((remote.clone(), protocol.clone()), notifications_sink);
                    }
                    this.event_streams.send(Event::NotificationStreamOpened {
                        remote,
                        protocol,
                        //TODO
                        info,
                    });
                }
                Poll::Ready(SwarmEvent::Behaviour(BehaviourOut::NotificationStreamReplaced {
                    remote,
                    protocol,
                    notifications_sink,
                })) => {
                    let mut peers_notifications_sinks = this.peers_notifications_sinks.lock();
                    if let Some(s) = peers_notifications_sinks.get_mut(&(remote, protocol)) {
                        *s = notifications_sink;
                    } else {
                        log::error!(
                            target: "sub-libp2p",
                            "NotificationStreamReplaced for non-existing substream"
                        );
                    }

                    // TODO: Notifications might have been lost as a result of the previous
                    // connection being dropped, and as a result it would be preferable to notify
                    // the users of this fact by simulating the substream being closed then
                    // reopened.
                    // The code below doesn't compile because `role` is unknown. Propagating the
                    // handshake of the secondary connections is quite an invasive change and
                    // would conflict with https://github.com/paritytech/substrate/issues/6403.
                    // Considering that dropping notifications is generally regarded as
                    // acceptable, this bug is at the moment intentionally left there and is
                    // intended to be fixed at the same time as
                    // https://github.com/paritytech/substrate/issues/6403.
                    /*this.event_streams.send(Event::NotificationStreamClosed {
                        remote,
                        engine_id,
                    });
                    this.event_streams.send(Event::NotificationStreamOpened {
                        remote,
                        engine_id,
                        role,
                    });*/
                }
                Poll::Ready(SwarmEvent::Behaviour(BehaviourOut::NotificationStreamClosed {
                    remote,
                    protocol,
                })) => {
                    if let Some(metrics) = this.metrics.as_ref() {
                        metrics
                            .notifications_streams_closed_total
                            .with_label_values(&[&protocol])
                            .inc();
                    }
                    this.event_streams.send(Event::NotificationStreamClosed {
                        remote: remote.clone(),
                        protocol: protocol.clone(),
                    });
                    {
                        let mut peers_notifications_sinks = this.peers_notifications_sinks.lock();
                        peers_notifications_sinks.remove(&(remote.clone(), protocol));
                    }
                }
                Poll::Ready(SwarmEvent::Behaviour(BehaviourOut::NotificationsReceived {
                    remote,
                    protocol,
                    messages,
                })) => {
                    if let Some(metrics) = this.metrics.as_ref() {
                        for message in &messages {
                            metrics
                                .notifications_sizes
                                .with_label_values(&["in", &protocol])
                                .observe(message.len() as f64);
                        }
                    }
                    this.event_streams.send(Event::NotificationsReceived {
                        remote,
                        protocol,
                        messages,
                    });
                }

                Poll::Ready(SwarmEvent::Behaviour(BehaviourOut::Dht(event, duration))) => {
                    if let Some(metrics) = this.metrics.as_ref() {
                        let query_type = match event {
                            DhtEvent::ValueFound(_) => "value-found",
                            DhtEvent::ValueNotFound(_) => "value-not-found",
                            DhtEvent::ValuePut(_) => "value-put",
                            DhtEvent::ValuePutFailed(_) => "value-put-failed",
                        };
                        metrics
                            .kademlia_query_duration
                            .with_label_values(&[query_type])
                            .observe(duration.as_secs_f64());
                    }

                    this.event_streams.send(Event::Dht(event));
                }
                Poll::Ready(SwarmEvent::ConnectionEstablished {
                    peer_id, endpoint, ..
                }) => {
                    trace!(target: "sub-libp2p", "Libp2p => Connected({:?})", peer_id);
                    if let Some(metrics) = this.metrics.as_ref() {
                        match endpoint {
                            ConnectedPoint::Dialer { .. } => metrics
                                .connections_opened_total
                                .with_label_values(&["out"])
                                .inc(),
                            ConnectedPoint::Listener { .. } => metrics
                                .connections_opened_total
                                .with_label_values(&["in"])
                                .inc(),
                        }
                    }
                }
                Poll::Ready(SwarmEvent::ConnectionClosed {
                    peer_id,
                    cause,
                    endpoint,
                    num_established,
                    ..
                }) => {
                    trace!(target: "sub-libp2p", "Libp2p => Disconnected({:?}, {:?})", peer_id, cause);
                    if let Some(metrics) = this.metrics.as_ref() {
                        let direction = match endpoint {
                            ConnectedPoint::Dialer { .. } => "out",
                            ConnectedPoint::Listener { .. } => "in",
                        };
                        let reason = match cause {
                            Some(ConnectionError::IO(_)) => "transport-error",
                            // Some(ConnectionError::Handler(NodeHandlerWrapperError::Handler(
                            //     EitherError::A(EitherError::A(EitherError::B(EitherError::A(
                            //         PingFailure::Timeout,
                            //     )))),
                            // ))) => "ping-timeout",
                            // Some(ConnectionError::Handler(NodeHandlerWrapperError::Handler(
                            //     EitherError::A(EitherError::A(EitherError::A(EitherError::A(
                            //         EitherError::A(EitherError::A(NotifsHandlerError::Legacy(
                            //             LegacyConnectionKillError,
                            //         ))),
                            //     )))),
                            // ))) => "force-closed",
                            // Some(ConnectionError::Handler(NodeHandlerWrapperError::Handler(
                            //     EitherError::A(EitherError::A(EitherError::A(EitherError::A(
                            //         EitherError::A(EitherError::A(
                            //             NotifsHandlerError::SyncNotificationsClogged,
                            //         )),
                            //     )))),
                            // ))) => "sync-notifications-clogged",
                            Some(ConnectionError::Handler(NodeHandlerWrapperError::Handler(_))) => {
                                "protocol-error"
                            }
                            Some(ConnectionError::Handler(
                                NodeHandlerWrapperError::KeepAliveTimeout,
                            )) => "keep-alive-timeout",
                            None => "actively-closed",
                        };
                        metrics
                            .connections_closed_total
                            .with_label_values(&[direction, reason])
                            .inc();

                        // `num_established` represents the number of *remaining* connections.
                        if num_established == 0 {
                            metrics.distinct_peers_connections_closed_total.inc();
                        }
                    }
                }
                Poll::Ready(SwarmEvent::NewListenAddr(addr)) => {
                    trace!(target: "sub-libp2p", "Libp2p => NewListenAddr({})", addr)
                }
                Poll::Ready(SwarmEvent::ExpiredListenAddr(addr)) => {
                    trace!(target: "sub-libp2p", "Libp2p => ExpiredListenAddr({})", addr)
                }
                Poll::Ready(SwarmEvent::UnreachableAddr {
                    peer_id,
                    address,
                    error,
                    ..
                }) => {
                    trace!(
                        target: "sub-libp2p", "Libp2p => Failed to reach {:?} through {:?}: {}",
                        peer_id,
                        address,
                        error,
                    );

                    if this.boot_node_ids.contains(&peer_id) {
                        if let PendingConnectionError::InvalidPeerId = error {
                            error!(
                                "💔 The bootnode you want to connect to at `{}` provided a different peer ID than the one you expect: `{}`.",
                                address,
                                peer_id,
                            );
                        }
                    }

                    if let Some(metrics) = this.metrics.as_ref() {
                        match error {
                            PendingConnectionError::ConnectionLimit(_) => metrics
                                .pending_connections_errors_total
                                .with_label_values(&["limit-reached"])
                                .inc(),
                            PendingConnectionError::InvalidPeerId => metrics
                                .pending_connections_errors_total
                                .with_label_values(&["invalid-peer-id"])
                                .inc(),
                            PendingConnectionError::Transport(_)
                            | PendingConnectionError::IO(_) => metrics
                                .pending_connections_errors_total
                                .with_label_values(&["transport-error"])
                                .inc(),
                        }
                    }
                }
                Poll::Ready(SwarmEvent::Dialing(peer_id)) => {
                    trace!(target: "sub-libp2p", "Libp2p => Dialing({:?})", peer_id)
                }
                Poll::Ready(SwarmEvent::IncomingConnection {
                    local_addr,
                    send_back_addr,
                }) => {
                    trace!(target: "sub-libp2p", "Libp2p => IncomingConnection({},{}))",
                           local_addr, send_back_addr);
                }
                Poll::Ready(SwarmEvent::IncomingConnectionError {
                    local_addr,
                    send_back_addr,
                    error,
                }) => {
                    trace!(target: "sub-libp2p", "Libp2p => IncomingConnectionError({},{}): {}",
                           local_addr, send_back_addr, error);
                }
                Poll::Ready(SwarmEvent::BannedPeer { peer_id, endpoint }) => {
                    trace!(target: "sub-libp2p", "Libp2p => BannedPeer({}). Connected via {:?}.",
                           peer_id, endpoint);
                }
                Poll::Ready(SwarmEvent::UnknownPeerUnreachableAddr { address, error }) => {
                    trace!(target: "sub-libp2p", "Libp2p => UnknownPeerUnreachableAddr({}): {}",
                           address, error)
                }
                Poll::Ready(SwarmEvent::ListenerClosed {
                    reason,
                    addresses: _,
                }) => {
                    warn!(target: "sub-libp2p", "Libp2p => ListenerClosed: {:?}", reason);
                }
                Poll::Ready(SwarmEvent::ListenerError { error }) => {
                    trace!(target: "sub-libp2p", "Libp2p => ListenerError: {}", error);
                }
            };
        }
        //
        // let num_connected_peers = this
        //     .network_service
        //     .user_protocol_mut()
        //     .num_connected_peers();

        // if let Some(metrics) = this.metrics.as_ref() {
        //     metrics
        //         .network_per_sec_bytes
        //         .with_label_values(&["in"])
        //         .set(this.service.bandwidth.average_download_per_sec());
        //     metrics
        //         .network_per_sec_bytes
        //         .with_label_values(&["out"])
        //         .set(this.service.bandwidth.average_upload_per_sec());
        //     metrics.peers_count.set(num_connected_peers as i64);
        // }

        Poll::Pending
    }
}

impl Unpin for NetworkWorker {}

/// The libp2p swarm, customized for our needs.
type Swarm = libp2p::swarm::Swarm<Behaviour>;

fn ensure_addresses_consistent_with_transport<'a>(
    addresses: impl Iterator<Item = &'a Multiaddr>,
    transport: &TransportConfig,
) -> Result<(), Error> {
    if matches!(transport, TransportConfig::MemoryOnly) {
        let addresses: Vec<_> = addresses
            .filter(|x| {
                x.iter()
                    .any(|y| !matches!(y, libp2p::core::multiaddr::Protocol::Memory(_)))
            })
            .cloned()
            .collect();

        if !addresses.is_empty() {
            return Err(Error::AddressesForAnotherTransport {
                transport: transport.clone(),
                addresses,
            });
        }
    } else {
        let addresses: Vec<_> = addresses
            .filter(|x| {
                x.iter()
                    .any(|y| matches!(y, libp2p::core::multiaddr::Protocol::Memory(_)))
            })
            .cloned()
            .collect();

        if !addresses.is_empty() {
            return Err(Error::AddressesForAnotherTransport {
                transport: transport.clone(),
                addresses,
            });
        }
    }

    Ok(())
}
