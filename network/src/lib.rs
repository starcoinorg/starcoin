// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

#[macro_use]
extern crate log;
#[macro_use]
extern crate prometheus;

pub mod helper;
mod network_metrics;
mod service;
pub mod service_ref;
#[cfg(test)]
mod tests;
pub mod worker;

pub use network_api::messages::*;
pub use service_ref::PeerMsgBroadcasterService;

pub use helper::get_unix_ts;

use network_p2p::PeerId;
pub use service::NetworkActorService;
pub use service_ref::NetworkServiceRef;
use std::borrow::Cow;
pub use worker::build_network_worker;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct NetworkMessage {
    pub peer_id: PeerId,
    pub protocol_name: Cow<'static, str>,
    pub data: Vec<u8>,
}
