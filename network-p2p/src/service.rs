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
use std::{borrow::Cow, collections::HashSet, fs, io, path::Path};

use futures::{
    channel::{mpsc, oneshot},
    prelude::*,
};
use libp2p::core::connection::ConnectionError;
use libp2p::swarm::{
    protocols_handler::NodeHandlerWrapperError, NetworkBehaviour, SwarmBuilder, SwarmEvent,
};
use libp2p::{kad::record, PeerId};
use log::{error, info, trace, warn};
use parking_lot::Mutex;
use peerset::{PeersetHandle, ReputationChange};
use types::peer_info::PeerInfo;

use crate::config::{Params, TransportConfig};
use crate::discovery::DiscoveryConfig;
use crate::metrics::Metrics;
use crate::net_error::Error;
use crate::network_state::{
    NetworkState, NotConnectedPeer as NetworkStateNotConnectedPeer, Peer as NetworkStatePeer,
};
use crate::protocol::event::Event;
use crate::protocol::{ChainInfo, Protocol};
use crate::{
    behaviour::{Behaviour, BehaviourOut},
    parse_addr, parse_str_addr, ConnectedPoint,
};
use crate::{config::NonReservedPeerMode, transport};
use crate::{Multiaddr, PROTOCOL_NAME};

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
    /// Channel that sends messages to the actual worker.
    to_worker: mpsc::UnboundedSender<ServiceToWorkerMsg>,
}

impl NetworkWorker {
    /// Creates the network service.
    ///
    /// Returns a `NetworkWorker` that implements `Future` and must be regularly polled in order
    /// for the network processing to advance. From it, you can extract a `NetworkService` using
    /// `worker.service()`. The `NetworkService` can be shared through the codebase.
    pub fn new(params: Params) -> Result<NetworkWorker, Error> {
        let (to_worker, from_worker) = mpsc::unbounded();

        if let Some(ref path) = params.network_config.net_config_path {
            fs::create_dir_all(Path::new(path))?;
        }

        // List of multiaddresses that we know in the network.
        let mut known_addresses = Vec::new();
        let mut bootnodes = Vec::new();
        let mut reserved_nodes = Vec::new();
        // 这个 boot_node_ids 一直为空，确认是否有必要。
        let boot_node_ids = HashSet::new();

        // Process the bootnodes.
        for bootnode in params.network_config.boot_nodes.iter() {
            match parse_addr(bootnode.clone()) {
                Ok((peer_id, addr)) => {
                    bootnodes.push(peer_id.clone());
                    known_addresses.push((peer_id, addr));
                }
                Err(_) => warn!(target: "sub-libp2p", "Not a valid bootnode address: {}", bootnode),
            }
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

        // Initialize the reserved peers.
        for reserved in params.network_config.reserved_nodes.iter() {
            if let Ok((peer_id, addr)) = parse_str_addr(reserved) {
                reserved_nodes.push(peer_id.clone());
                known_addresses.push((peer_id, addr));
            } else {
                warn!(target: "sub-libp2p", "Not a valid reserved node address: {}", reserved);
            }
        }

        let peerset_config = peerset::PeersetConfig {
            in_peers: params.network_config.in_peers,
            out_peers: params.network_config.out_peers,
            bootnodes,
            reserved_only: params.network_config.non_reserved_mode == NonReservedPeerMode::Deny,
            reserved_nodes,
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
        let (mut protocol, peerset_handle) = Protocol::new(
            peerset_config,
            params.protocol_id.clone(),
            chain_info,
            boot_node_ids,
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

            protocol.register_notifications_protocol(PROTOCOL_NAME);
            for protocol_name in params.network_config.protocols {
                protocol.register_notifications_protocol(protocol_name);
            }

            let behaviour = futures::executor::block_on(Behaviour::new(
                protocol,
                user_agent,
                local_public,
                discovery_config,
            ));

            let (transport, bandwidth) = {
                let (config_mem, config_wasm, flowctrl) = match params.network_config.transport {
                    TransportConfig::MemoryOnly => (true, None, false),
                    TransportConfig::Normal {
                        wasm_external_transport,
                        use_yamux_flow_control,
                        ..
                    } => (false, wasm_external_transport, use_yamux_flow_control),
                };
                transport::build_transport(local_identity, config_mem, config_wasm, flowctrl)
            };
            let builder = SwarmBuilder::new(transport, behaviour, local_peer_id.clone());
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

        let service = Arc::new(NetworkService {
            bandwidth,
            external_addresses,
            num_connected,
            is_major_syncing,
            peerset: peerset_handle,
            local_peer_id,
            to_worker,
        });

        Ok(NetworkWorker {
            network_service: swarm,
            service,
            from_worker,
            event_streams: Vec::new(),
            metrics: Metrics::register().ok(),
        })
    }

    /// Returns the downloaded bytes per second averaged over the past few seconds.
    pub fn average_download_per_sec(&self) -> u64 {
        self.service.bandwidth.average_download_per_sec()
    }

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
            let list = swarm
                .known_peers()
                .filter(|p| open.iter().all(|n| n != *p))
                .cloned()
                .collect::<Vec<_>>();
            list.into_iter()
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
            average_download_per_sec: self.service.bandwidth.average_download_per_sec(),
            average_upload_per_sec: self.service.bandwidth.average_upload_per_sec(),
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
    pub fn connected_peers(&mut self) -> impl Iterator<Item = &PeerId> {
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
        protocol_name: Cow<'static, [u8]>,
        message: Vec<u8>,
    ) {
        let _ = self
            .to_worker
            .unbounded_send(ServiceToWorkerMsg::WriteNotification {
                target,
                protocol_name,
                message,
            });
    }

    pub async fn broadcast_message(&self, protocol_name: Cow<'static, [u8]>, message: Vec<u8>) {
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
    pub fn event_stream(&self) -> impl Stream<Item = Event> {
        // Note: when transitioning to stable futures, remove the `Error` entirely
        let (tx, rx) = mpsc::unbounded();
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
    pub fn register_notifications_protocol(&self, protocol_name: impl Into<Cow<'static, [u8]>>) {
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

    pub async fn exist_notif_proto(&self, protocol_name: Cow<'static, [u8]>) -> bool {
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

/// Messages sent from the `NetworkService` to the `NetworkWorker`.
///
/// Each entry corresponds to a method of `NetworkService`.
enum ServiceToWorkerMsg {
    GetValue(record::Key),
    PutValue(record::Key, Vec<u8>),
    AddKnownAddress(PeerId, Multiaddr),
    EventStream(mpsc::UnboundedSender<Event>),
    WriteNotification {
        message: Vec<u8>,
        protocol_name: Cow<'static, [u8]>,
        target: PeerId,
    },
    RegisterNotifProtocol {
        protocol_name: Cow<'static, [u8]>,
    },
    DisconnectPeer(PeerId),
    IsConnected(PeerId, oneshot::Sender<bool>),
    ConnectedPeers(oneshot::Sender<HashSet<PeerId>>),
    SelfInfo(Box<PeerInfo>),
    AddressByPeerID(PeerId, oneshot::Sender<Vec<Multiaddr>>),
    ExistNotifProtocol {
        protocol_name: Cow<'static, [u8]>,
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
    event_streams: Vec<mpsc::UnboundedSender<Event>>,
    /// Prometheus network metrics.
    metrics: Option<Metrics>,
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
                ServiceToWorkerMsg::WriteNotification {
                    message,
                    protocol_name,
                    target,
                } => this.network_service.user_protocol_mut().write_notification(
                    target,
                    protocol_name,
                    message,
                ),
                ServiceToWorkerMsg::RegisterNotifProtocol { protocol_name } => {
                    let events = this
                        .network_service
                        .user_protocol_mut()
                        .register_notifications_protocol(protocol_name);
                    for event in events {
                        this.event_streams
                            .retain(|sender| sender.unbounded_send(event.clone()).is_ok());
                    }
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

        loop {
            // Process the next action coming from the network.
            let next_event = this.network_service.next_event();
            futures::pin_mut!(next_event);
            let poll_value = next_event.poll_unpin(cx);

            match poll_value {
                Poll::Pending => break,
                Poll::Ready(SwarmEvent::Behaviour(BehaviourOut::Event(ev))) => this
                    .event_streams
                    .retain(|sender| sender.unbounded_send(ev.clone()).is_ok()),
                Poll::Ready(SwarmEvent::Behaviour(BehaviourOut::RandomKademliaStarted(_))) => {}
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
                    ..
                }) => {
                    trace!(target: "sub-libp2p", "Libp2p => Disconnected({:?}, {:?})", peer_id, cause);
                    if let Some(metrics) = this.metrics.as_ref() {
                        let dir = match endpoint {
                            ConnectedPoint::Dialer { .. } => "out",
                            ConnectedPoint::Listener { .. } => "in",
                        };

                        match cause {
                            ConnectionError::IO(_) => metrics
                                .connections_closed_total
                                .with_label_values(&[dir, "transport-error"])
                                .inc(),
                            ConnectionError::Handler(NodeHandlerWrapperError::Handler(_)) => {
                                metrics
                                    .connections_closed_total
                                    .with_label_values(&[dir, "protocol-error"])
                                    .inc()
                            }
                            ConnectionError::Handler(NodeHandlerWrapperError::KeepAliveTimeout) => {
                                metrics
                                    .connections_closed_total
                                    .with_label_values(&[dir, "keep-alive-timeout"])
                                    .inc()
                            }
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
                    trace!(target: "sub-libp2p", "Libp2p => Failed to reach {:?} through {:?}: {}", peer_id, address, error)
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

        let num_connected_peers = this
            .network_service
            .user_protocol_mut()
            .num_connected_peers();

        if let Some(metrics) = this.metrics.as_ref() {
            metrics
                .network_per_sec_bytes
                .with_label_values(&["in"])
                .set(this.service.bandwidth.average_download_per_sec());
            metrics
                .network_per_sec_bytes
                .with_label_values(&["out"])
                .set(this.service.bandwidth.average_upload_per_sec());
            metrics.peers_count.set(num_connected_peers as i64);
        }

        Poll::Pending
    }
}

impl Unpin for NetworkWorker {}

/// The libp2p swarm, customized for our needs.
type Swarm = libp2p::swarm::Swarm<Behaviour>;
