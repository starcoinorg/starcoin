// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::behaviour::BehaviourOut::Event;
use crate::config::{NonReservedPeerMode, TransportConfig};
use crate::{
    behaviour::{Behaviour, BehaviourOut},
    parse_str_addr, transport, NetworkConfiguration, NetworkState, NetworkStateNotConnectedPeer,
    NetworkStatePeer, ProtocolId,
};
use bytes::BytesMut;
use fnv::FnvHashMap;
use futures::{prelude::*, Stream};
use libp2p::Swarm;
use libp2p::{
    core::{
        muxing::StreamMuxerBox, nodes::Substream, transport::boxed::Boxed, ConnectedPoint,
        Multiaddr, PeerId,
    },
    swarm::NetworkBehaviour,
};
use log::{debug, info, warn};
use sc_peerset::{Peerset, PeersetConfig};
use std::{fs, io, io::Error as IoError, path::Path, sync::Arc, time::Duration};

/// Starts the libp2p service.
///
/// Returns a stream that must be polled regularly in order for the networking to function.
pub fn start_service(
    protocol: impl Into<ProtocolId>,
    config: NetworkConfiguration,
) -> Result<(Service, sc_peerset::PeersetHandle), IoError> {
    if let Some(ref path) = config.net_config_path {
        fs::create_dir_all(Path::new(path))?;
    }

    // List of multiaddresses that we know in the network.
    let mut known_addresses = Vec::new();
    let mut bootnodes = Vec::new();
    let mut reserved_nodes = Vec::new();

    // Process the bootnodes.
    for bootnode in config.boot_nodes.iter() {
        match parse_str_addr(bootnode) {
            Ok((peer_id, addr)) => {
                bootnodes.push(peer_id.clone());
                known_addresses.push((peer_id, addr));
            }
            Err(_) => warn!(target: "sg-libp2p", "Not a valid bootnode address: {}", bootnode),
        }
    }

    // Initialize the reserved peers.
    for reserved in config.reserved_nodes.iter() {
        if let Ok((peer_id, addr)) = parse_str_addr(reserved) {
            reserved_nodes.push(peer_id.clone());
            known_addresses.push((peer_id, addr));
        } else {
            warn!(target: "sg-libp2p", "Not a valid reserved node address: {}", reserved);
        }
    }

    // Build the peerset.
    let (peerset, peerset_handle) = Peerset::from_config(PeersetConfig {
        in_peers: config.in_peers,
        out_peers: config.out_peers,
        bootnodes,
        reserved_only: config.non_reserved_mode == NonReservedPeerMode::Deny,
        reserved_nodes,
    });

    // Private and public keys configuration.
    let local_identity = config.node_key.clone().into_keypair()?;
    let local_public = local_identity.public();
    let local_peer_id = local_public.clone().into_peer_id();
    info!(target: "sg-libp2p", "Local node identity is: {}", local_peer_id.to_base58());

    // Build the swarm.
    let (mut swarm, bandwidth) = {
        let user_agent = format!("{} ({})", config.client_version, config.node_name);
        let behaviour = Behaviour::new(
            protocol,
            user_agent,
            local_public,
            known_addresses,
            match config.transport {
                TransportConfig::MemoryOnly => false,
                TransportConfig::Normal { enable_mdns, .. } => enable_mdns,
            },
            match config.transport {
                TransportConfig::MemoryOnly => false,
                TransportConfig::Normal {
                    allow_private_ipv4, ..
                } => allow_private_ipv4,
            },
            peerset,
        );
        let (transport, bandwidth) = {
            let (config_mem, config_wasm) = match config.transport {
                TransportConfig::MemoryOnly => (true, None),
                TransportConfig::Normal {
                    wasm_external_transport,
                    ..
                } => (false, wasm_external_transport),
            };
            transport::build_transport(local_identity, config_mem, config_wasm)
        };
        (
            Swarm::new(transport, behaviour, local_peer_id.clone()),
            bandwidth,
        )
    };

    // Listen on multiaddresses.
    for addr in &config.listen_addresses {
        if let Err(err) = Swarm::listen_on(&mut swarm, addr.clone()) {
            warn!(target: "sg-libp2p", "Can't listen on {} because: {:?}", addr, err)
        }
    }

    // Add external addresses.
    for addr in &config.public_addresses {
        Swarm::add_external_address(&mut swarm, addr.clone());
    }

    let service = Service {
        swarm,
        bandwidth,
        nodes_info: Default::default(),
        injected_events: Vec::new(),
    };

    Ok((service, peerset_handle))
}

/// Event produced by the service.
#[derive(Debug)]
pub enum ServiceEvent {
    /// A custom protocol substream has been opened with a node.
    OpenedCustomProtocol {
        /// Identity of the node.
        peer_id: PeerId,
        /// Version of the protocol that was opened.
        version: u8,
        /// Node debug info
        debug_info: String,
    },

    /// A custom protocol substream has been closed.
    ClosedCustomProtocol {
        /// Identity of the node.
        peer_id: PeerId,
        /// Node debug info
        debug_info: String,
    },

    /// Receives a message on a custom protocol stream.
    CustomMessage {
        /// Identity of the node.
        peer_id: PeerId,
        /// Message that has been received.
        message: BytesMut,
    },

    /// The substream with a node is clogged. We should avoid sending data to it if possible.
    Clogged {
        /// Index of the node.
        peer_id: PeerId,
        /// Copy of the messages that are within the buffer, for further diagnostic.
        messages: Vec<Vec<u8>>,
    },
}

/// Network service. Must be polled regularly in order for the networking to work.
pub struct Service {
    /// Stream of events of the swarm.
    swarm: libp2p::swarm::Swarm<
        Boxed<(PeerId, StreamMuxerBox), io::Error>,
        Behaviour<Substream<StreamMuxerBox>>,
    >,

    /// Bandwidth logging system. Can be queried to know the average bandwidth consumed.
    bandwidth: Arc<transport::BandwidthSinks>,

    /// Information about all the nodes we're connected to.
    nodes_info: FnvHashMap<PeerId, NodeInfo>,

    /// Events to produce on the Stream.
    injected_events: Vec<ServiceEvent>,
}

/// Information about a node we're connected to.
#[derive(Debug)]
struct NodeInfo {
    /// How we're connected to the node.
    endpoint: ConnectedPoint,
    /// Version reported by the remote, or `None` if unknown.
    client_version: Option<String>,
    /// Latest ping time with this node.
    latest_ping: Option<Duration>,
}

impl Service {
    /// Returns a struct containing tons of useful information about the network.

    pub fn is_open(&self, peer_id: &PeerId) -> bool {
        self.swarm.is_open(peer_id)
    }

    pub fn state(&mut self) -> NetworkState {
        let connected_peers = {
            let swarm = &mut self.swarm;
            self.nodes_info
                .iter()
                .map(move |(peer_id, info)| {
                    let known_addresses =
                        NetworkBehaviour::addresses_of_peer(&mut **swarm, peer_id)
                            .into_iter()
                            .collect();

                    (
                        peer_id.to_base58(),
                        NetworkStatePeer {
                            endpoint: info.endpoint.clone().into(),
                            version_string: info.client_version.clone(),
                            latest_ping_time: info.latest_ping,
                            enabled: swarm.is_enabled(&peer_id),
                            open: swarm.is_open(&peer_id),
                            known_addresses,
                        },
                    )
                })
                .collect()
        };

        let not_connected_peers = {
            let swarm = &mut self.swarm;
            let nodes_info = &self.nodes_info;
            let list = swarm
                .known_peers()
                .filter(|p| !nodes_info.contains_key(p))
                .cloned()
                .collect::<Vec<_>>();
            list.into_iter()
                .map(move |peer_id| {
                    (
                        peer_id.to_base58(),
                        NetworkStateNotConnectedPeer {
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
            peer_id: Swarm::local_peer_id(&self.swarm).to_base58(),
            listened_addresses: Swarm::listeners(&self.swarm).cloned().collect(),
            external_addresses: Swarm::external_addresses(&self.swarm).cloned().collect(),
            average_download_per_sec: self.bandwidth.average_download_per_sec(),
            average_upload_per_sec: self.bandwidth.average_upload_per_sec(),
            connected_peers,
            not_connected_peers,
            peerset: self.swarm.peerset_debug_info(),
        }
    }

    /// Returns an iterator that produces the list of addresses we're listening on.
    #[inline]
    pub fn listeners(&self) -> impl Iterator<Item = &Multiaddr> {
        Swarm::listeners(&self.swarm)
    }

    /// Returns the downloaded bytes per second averaged over the past few seconds.
    #[inline]
    pub fn average_download_per_sec(&self) -> u64 {
        self.bandwidth.average_download_per_sec()
    }

    /// Returns the uploaded bytes per second averaged over the past few seconds.
    #[inline]
    pub fn average_upload_per_sec(&self) -> u64 {
        self.bandwidth.average_upload_per_sec()
    }

    /// Returns the peer id of the local node.
    #[inline]
    pub fn peer_id(&self) -> &PeerId {
        Swarm::local_peer_id(&self.swarm)
    }

    /// Returns the list of all the peers we are connected to.
    #[inline]
    pub fn connected_peers<'a>(&'a self) -> impl Iterator<Item = &'a PeerId> + 'a {
        self.nodes_info.keys()
    }

    /// Returns the way we are connected to a node.
    #[inline]
    pub fn node_endpoint(&self, peer_id: &PeerId) -> Option<&ConnectedPoint> {
        self.nodes_info.get(peer_id).map(|info| &info.endpoint)
    }

    /// Returns the client version reported by a node.
    pub fn node_client_version(&self, peer_id: &PeerId) -> Option<&str> {
        self.nodes_info
            .get(peer_id)
            .and_then(|info| info.client_version.as_ref().map(|s| &s[..]))
    }

    /// Sends a message to a peer using the custom protocol.
    ///
    /// Has no effect if the connection to the node has been closed, or if the node index is
    /// invalid.
    pub fn send_custom_message(&mut self, peer_id: &PeerId, message: Vec<u8>) {
        self.swarm.send_custom_message(peer_id, message);
    }

    /// Disconnects a peer.
    ///
    /// This is asynchronous and will not immediately close the peer.
    /// Corresponding closing events will be generated once the closing actually happens.
    pub fn drop_node(&mut self, peer_id: &PeerId) {
        if let Some(info) = self.nodes_info.get(peer_id) {
            debug!(target: "sg-libp2p", "Dropping {:?} on purpose ({:?}, {:?})",
                   peer_id, info.endpoint, info.client_version);
            self.swarm.drop_node(peer_id);
        }
    }

    /// Adds a hard-coded address for the given peer, that never expires.
    pub fn add_known_address(&mut self, peer_id: PeerId, addr: Multiaddr) {
        self.swarm.add_known_address(peer_id, addr)
    }

    /// Get debug info for a given peer.
    pub fn peer_debug_info(&self, who: &PeerId) -> String {
        if let Some(info) = self.nodes_info.get(who) {
            format!(
                "{:?} (version: {:?}) through {:?}",
                who, info.client_version, info.endpoint
            )
        } else {
            "unknown".to_string()
        }
    }

    /// Polls for what happened on the network.
    fn poll_swarm(&mut self) -> Poll<Option<ServiceEvent>, IoError> {
        loop {
            match self.swarm.poll() {
                Ok(Async::Ready(Some(Event(_)))) => {
                    //TODO: handle dht event
                }
                Ok(Async::Ready(Some(BehaviourOut::CustomProtocolOpen {
                    peer_id,
                    version,
                    endpoint,
                }))) => {
                    self.nodes_info.insert(
                        peer_id.clone(),
                        NodeInfo {
                            endpoint,
                            client_version: None,
                            latest_ping: None,
                        },
                    );
                    let debug_info = self.peer_debug_info(&peer_id);
                    break Ok(Async::Ready(Some(ServiceEvent::OpenedCustomProtocol {
                        peer_id,
                        version,
                        debug_info,
                    })));
                }
                Ok(Async::Ready(Some(BehaviourOut::CustomProtocolClosed { peer_id, .. }))) => {
                    let debug_info = self.peer_debug_info(&peer_id);
                    self.nodes_info.remove(&peer_id);
                    break Ok(Async::Ready(Some(ServiceEvent::ClosedCustomProtocol {
                        peer_id,
                        debug_info,
                    })));
                }
                Ok(Async::Ready(Some(BehaviourOut::CustomMessage { peer_id, message }))) => {
                    break Ok(Async::Ready(Some(ServiceEvent::CustomMessage {
                        peer_id,
                        message,
                    })));
                }
                Ok(Async::Ready(Some(BehaviourOut::Clogged { peer_id, messages }))) => {
                    break Ok(Async::Ready(Some(ServiceEvent::Clogged {
                        peer_id,
                        messages,
                    })));
                }
                Ok(Async::Ready(Some(BehaviourOut::Identified { peer_id, info }))) => {
                    // Contrary to the other events, this one can happen even on nodes which don't
                    // have any open custom protocol slot. Therefore it is not necessarily in the
                    // list.
                    if let Some(n) = self.nodes_info.get_mut(&peer_id) {
                        n.client_version = Some(info.agent_version);
                    }
                }
                Ok(Async::Ready(Some(BehaviourOut::PingSuccess { peer_id, ping_time }))) => {
                    // Contrary to the other events, this one can happen even on nodes which don't
                    // have any open custom protocol slot. Therefore it is not necessarily in the
                    // list.
                    if let Some(n) = self.nodes_info.get_mut(&peer_id) {
                        n.latest_ping = Some(ping_time);
                    }
                }
                Ok(Async::NotReady) => break Ok(Async::NotReady),
                Ok(Async::Ready(None)) => unreachable!("The Swarm stream never ends"),
                Err(_) => unreachable!("The Swarm never errors"),
            }
        }
    }
}

impl Stream for Service {
    type Item = ServiceEvent;
    type Error = IoError;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        if !self.injected_events.is_empty() {
            return Ok(Async::Ready(Some(self.injected_events.remove(0))));
        }

        match self.poll_swarm()? {
            Async::Ready(value) => return Ok(Async::Ready(value)),
            Async::NotReady => (),
        }

        // The only way we reach this is if we went through all the `NotReady` paths above,
        // ensuring the current task is registered everywhere.
        Ok(Async::NotReady)
    }
}
