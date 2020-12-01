// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

#[macro_use]
extern crate log;
#[macro_use]
extern crate prometheus;

pub mod helper;
pub mod net;
#[cfg(test)]
mod net_test;
pub mod network;
mod network_metrics;

pub use network::PeerMsgBroadcasterService;
pub use network_api::messages::*;

pub use helper::get_unix_ts;

pub use net::{build_network_service, SNetworkService};
pub use network::NetworkAsyncService;
use network_p2p::PeerId;
use std::borrow::Cow;

#[cfg(test)]
pub use net::NetworkInner;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct NetworkMessage {
    pub peer_id: PeerId,
    pub protocol_name: Cow<'static, str>,
    pub data: Vec<u8>,
}
