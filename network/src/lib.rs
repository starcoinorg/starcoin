// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod messages;
mod network;
mod node_discovery;
mod peer;
mod peer_manager;

pub use messages::*;
pub use network::NetworkActor;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
