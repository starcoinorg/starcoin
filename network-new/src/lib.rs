// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

#[macro_use]
extern crate log;

mod helper;
mod message_processor;
mod messages;
mod net;
pub mod network;
pub mod sync_messages;

pub use messages::*;
pub use network::NetworkActor;

pub use helper::{
    convert_account_address_to_peer_id, convert_peer_id_to_account_address, get_unix_ts,
};

use anyhow::{Error, Result};
pub use messages::{
    PeerEvent, PeerMessage, RPCMessage, RPCRequest, RPCResponse, RpcRequestMessage,
};
pub use net::{build_network_service, NetworkComponent, NetworkService};
pub use network::NetworkAsyncService;
pub use network_p2p::PeerId;
use types::system_events::SystemEvents;
