// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod messages;
mod network;
mod node_discovery;
mod peer;
mod peer_manager;
mod transport;
mod utils;
mod discovery;
mod debug_info;
mod config;
mod protocol;

pub use messages::*;
pub use network::NetworkActor;
pub use libp2p::{Multiaddr, PeerId};

/// Extension trait for `NetworkBehaviour` that also accepts discovering nodes.
pub trait DiscoveryNetBehaviour {
    /// Notify the protocol that we have learned about the existence of nodes.
    ///
    /// Can (or most likely will) be called multiple times with the same `PeerId`s.
    ///
    /// Also note that there is no notification for expired nodes. The implementer must add a TTL
    /// system, or remove nodes that will fail to reach.
    fn add_discovered_nodes(&mut self, nodes: impl Iterator<Item = PeerId>);
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
