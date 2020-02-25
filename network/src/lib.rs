// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

#[macro_use]
extern crate log;

mod helper;
mod messages;
mod net;
mod network;
mod node_discovery;
mod peer;
mod peer_manager;

pub use messages::*;
pub use network::NetworkActor;

pub use helper::{
    convert_account_address_to_peer_id, convert_peer_id_to_account_address, get_unix_ts,
};

pub use net::{build_network_service, NetworkComponent, NetworkService};
pub use network_libp2p::PeerId;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
