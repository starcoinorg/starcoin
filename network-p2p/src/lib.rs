// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

extern crate prometheus;
#[macro_use]
extern crate starcoin_metrics;

pub use crate::protocol::event::{DhtEvent, Event};
pub use crate::protocol::generic_proto::GenericProtoOut;
pub use crate::service::{NetworkService, NetworkWorker};
pub use config::{NetworkConfiguration, NodeKeyConfig, Params, ProtocolId, Secret};
pub use libp2p::{
    core::{
        ConnectedPoint, {identity, multiaddr, Multiaddr, PeerId, PublicKey},
    },
    multiaddr as build_multiaddr,
};

//TODO change to private
pub mod behaviour;
pub mod config;
//TODO change to private
pub mod discovery;
mod errors;
mod metrics;
mod network_state;
mod out_events;
mod peer_info;
//TODO change to private
#[allow(clippy::result_unit_err)]
pub mod protocol;
#[allow(clippy::single_match)]
mod request_responses;
mod service;
#[cfg(test)]
mod service_test;
mod transport;
mod utils;

const MAX_CONNECTIONS_PER_PEER: usize = 2;
/// The maximum number of concurrent established connections that were incoming.
const MAX_CONNECTIONS_ESTABLISHED_INCOMING: u32 = 10_000;

trait DiscoveryNetBehaviour {
    /// Notify the protocol that we have learned about the existence of nodes.
    ///
    /// Can (or most likely will) be called multiple times with the same `PeerId`s.
    ///
    /// Also note that there is no notification for expired nodes. The implementer must add a TTL
    /// system, or remove nodes that will fail to reach.
    fn add_discovered_nodes(&mut self, nodes: impl Iterator<Item = PeerId>);
}
