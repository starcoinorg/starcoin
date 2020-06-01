// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

#[macro_use]
extern crate log;
#[macro_use]
extern crate prometheus;

mod helper;
mod message_processor;
mod net;
mod net_test;
pub mod network;
mod network_metrics;

pub use network::NetworkActor;
pub use network_api::messages::*;

pub use helper::get_unix_ts;

pub use net::{build_network_service, SNetworkService};
pub use network::NetworkAsyncService;
pub use network_p2p::PeerId;

use anyhow::*;
use parity_codec::{Decode, Encode};
use std::borrow::Cow;
use types::account_address::AccountAddress;

#[derive(Clone, Hash, Debug)]
pub struct InnerMessage {
    pub peer_id: AccountAddress,
    pub msg: Message,
}

#[derive(Debug, PartialEq, Hash, Eq, Clone, Encode, Decode)]
pub enum Message {
    ACK(u128),
    Payload(PayloadMsg),
}

#[derive(Debug, PartialEq, Hash, Eq, Clone, Encode, Decode)]
pub struct PayloadMsg {
    pub id: u128,
    pub data: Vec<u8>,
}

impl Message
where
    Self: Decode + Encode,
{
    pub fn into_bytes(self) -> Vec<u8> {
        self.encode()
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        Decode::decode(&mut &bytes[..]).ok_or_else(|| anyhow!("decode data error"))
    }
}

impl Message {
    pub fn new_ack(message_id: u128) -> Message {
        Message::ACK(message_id)
    }

    pub fn new_payload(data: Vec<u8>) -> (Message, u128) {
        let message_id = get_unix_ts();
        (
            Message::Payload(PayloadMsg {
                id: message_id,
                data,
            }),
            message_id,
        )
    }
    pub fn new_message(data: Vec<u8>) -> Message {
        Message::Payload(PayloadMsg { id: 0, data })
    }

    pub fn into_payload(self) -> Option<Vec<u8>> {
        match self {
            Message::Payload(p) => Some(p.data),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct NetworkMessage {
    pub peer_id: PeerId,
    pub protocol_name: Cow<'static, [u8]>,
    pub data: Vec<u8>,
}
