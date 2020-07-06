// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

#[macro_use]
extern crate log;
extern crate prometheus;
#[macro_use]
extern crate starcoin_metrics;

pub use crate::protocol::event::Event;
pub use crate::protocol::generic_proto::GenericProtoOut;
pub use crate::service::{NetworkService, NetworkWorker};
pub use config::{NetworkConfiguration, NodeKeyConfig, Params, ProtocolId, Secret};
pub use libp2p::{
    core::{
        ConnectedPoint, {identity, multiaddr, Multiaddr, PeerId, PublicKey},
    },
    multiaddr as build_multiaddr,
};

mod behaviour;
pub mod config;
mod debug_info;
mod discovery;
mod metrics;
mod net_error;
mod network_state;
mod protocol;
mod service;
mod service_test;
mod transport;
mod utils;

use serde::Serialize;
use std::{
    collections::{HashMap, HashSet},
    error, fmt,
    time::Duration,
};

const MAX_CONNECTIONS_PER_PEER: usize = 2;
pub const PROTOCOL_NAME: &[u8] = b"/starcoin/consensus/1";

trait DiscoveryNetBehaviour {
    /// Notify the protocol that we have learned about the existence of nodes.
    ///
    /// Can (or most likely will) be called multiple times with the same `PeerId`s.
    ///
    /// Also note that there is no notification for expired nodes. The implementer must add a TTL
    /// system, or remove nodes that will fail to reach.
    fn add_discovered_nodes(&mut self, nodes: impl Iterator<Item = PeerId>);
}

/// Parses a string address and returns the component, if valid.
pub fn parse_str_addr(addr_str: &str) -> Result<(PeerId, Multiaddr), ParseErr> {
    let mut addr: Multiaddr = addr_str.parse()?;

    let who = match addr.pop() {
        Some(multiaddr::Protocol::P2p(key)) => {
            PeerId::from_multihash(key).map_err(|_| ParseErr::InvalidPeerId)?
        }
        _ => return Err(ParseErr::PeerIdMissing),
    };

    Ok((who, addr))
}

/// Splits a Multiaddress into a Multiaddress and PeerId.
pub fn parse_addr(mut addr: Multiaddr) -> Result<(PeerId, Multiaddr), ParseErr> {
    let who = match addr.pop() {
        Some(multiaddr::Protocol::P2p(key)) => {
            PeerId::from_multihash(key).map_err(|_| ParseErr::InvalidPeerId)?
        }
        _ => return Err(ParseErr::PeerIdMissing),
    };

    Ok((who, addr))
}

/// Error that can be generated by `parse_str_addr`.
#[derive(Debug)]
pub enum ParseErr {
    /// Error while parsing the multiaddress.
    MultiaddrParse(multiaddr::Error),
    /// Multihash of the peer ID is invalid.
    InvalidPeerId,
    /// The peer ID is missing from the address.
    PeerIdMissing,
}

impl fmt::Display for ParseErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseErr::MultiaddrParse(err) => write!(f, "{}", err),
            ParseErr::InvalidPeerId => write!(f, "Peer id at the end of the address is invalid"),
            ParseErr::PeerIdMissing => write!(f, "Peer id is missing from the address"),
        }
    }
}

impl error::Error for ParseErr {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            ParseErr::MultiaddrParse(err) => Some(err),
            ParseErr::InvalidPeerId => None,
            ParseErr::PeerIdMissing => None,
        }
    }
}

impl From<multiaddr::Error> for ParseErr {
    fn from(err: multiaddr::Error) -> ParseErr {
        ParseErr::MultiaddrParse(err)
    }
}

/// Returns general information about the networking.
///
/// Meant for general diagnostic purposes.
///
/// **Warning**: This API is not stable.
#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkState {
    /// PeerId of the local node.
    pub peer_id: String,
    /// List of addresses the node is currently listening on.
    pub listened_addresses: HashSet<Multiaddr>,
    /// List of addresses the node knows it can be reached as.
    pub external_addresses: HashSet<Multiaddr>,
    /// List of node we're connected to.
    pub connected_peers: HashMap<String, NetworkStatePeer>,
    /// List of node that we know of but that we're not connected to.
    pub not_connected_peers: HashMap<String, NetworkStateNotConnectedPeer>,
    /// Downloaded bytes per second averaged over the past few seconds.
    pub average_download_per_sec: u64,
    /// Uploaded bytes per second averaged over the past few seconds.
    pub average_upload_per_sec: u64,
    /// State of the peerset manager.
    pub peerset: serde_json::Value,
}

#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkStatePeer {
    /// How we are connected to the node.
    pub endpoint: NetworkStatePeerEndpoint,
    /// Node information, as provided by the node itself. Can be empty if not known yet.
    pub version_string: Option<String>,
    /// Latest ping duration with this node.
    pub latest_ping_time: Option<Duration>,
    /// If true, the peer is "enabled", which means that we try to open stargate related protocols
    /// with this peer. If false, we stick to Kademlia and/or other network-only protocols.
    pub enabled: bool,
    /// If true, the peer is "open", which means that we have a stargate related protocol
    /// with this peer.
    pub open: bool,
    /// List of addresses known for this node.
    pub known_addresses: HashSet<Multiaddr>,
}

#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkStateNotConnectedPeer {
    /// List of addresses known for this node.
    pub known_addresses: HashSet<Multiaddr>,
}

#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum NetworkStatePeerEndpoint {
    /// We are dialing the given address.
    Dialing(Multiaddr),
    /// We are listening.
    Listening {
        /// Address we're listening on that received the connection.
        local_addr: Multiaddr,
        /// Address data is sent back to.
        send_back_addr: Multiaddr,
    },
}

impl From<ConnectedPoint> for NetworkStatePeerEndpoint {
    fn from(endpoint: ConnectedPoint) -> Self {
        match endpoint {
            ConnectedPoint::Dialer { address } => NetworkStatePeerEndpoint::Dialing(address),
            ConnectedPoint::Listener {
                local_addr,
                send_back_addr,
            } => NetworkStatePeerEndpoint::Listening {
                local_addr,
                send_back_addr,
            },
        }
    }
}
