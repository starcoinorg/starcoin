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
mod message_processor;

pub use messages::*;
pub use network::NetworkActor;

pub use helper::{
    convert_account_address_to_peer_id, convert_peer_id_to_account_address, get_unix_ts,
};

pub use net::{build_network_service, NetworkComponent, NetworkService};
pub use network_libp2p::PeerId;
pub use messages::RPCMessage;
use types::{system_events::SystemEvents};
use anyhow::{Error, Result};
use types::account_address::AccountAddress;
use crypto::hash::HashValue;

#[async_trait::async_trait]
pub trait NetWorkAsyncService: Clone + std::marker::Unpin {
    async fn send_system_event(self,peer_id:AccountAddress, event: SystemEvents) -> Result<bool>;

    async fn broadcast_system_event(self,event: SystemEvents) -> Result<bool>;

    async fn send_request(
        self,
        peer_id:AccountAddress,
        message:RPCMessage,
    ) -> Result<RPCMessage>;

    async fn response_for(self, id: HashValue,response:RPCMessage);
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
