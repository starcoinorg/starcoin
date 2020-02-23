// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::helper::get_unix_ts;
use libra_crypto::{
    hash::{CryptoHash, CryptoHasher, TestOnlyHasher},
    HashValue,
};
use types::account_address::AccountAddress;
use parity_codec::{Decode, Encode};
use actix::prelude::*;
use anyhow::Result;
use types::transaction::SignedUserTransaction;

#[derive(Message)]
#[rtype(result = "u64")]
pub struct GetCounterMessage {}

/// message from peer
#[derive(Message)]
#[rtype(result = "Result<()>")]
pub enum PeerMessage {
    UserTransaction(SignedUserTransaction),
}

#[derive(Clone, Debug)]
pub struct InnerMessage {
    pub peer_id: AccountAddress,
    pub msg: Message,
}

impl CryptoHash for InnerMessage {
    type Hasher = TestOnlyHasher;

    fn hash(&self) -> HashValue {
        let mut state = Self::Hasher::default();
        state.write(&self.msg.clone().into_bytes());
        state.finish()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Encode, Decode)]
pub enum Message {
    ACK(u128),
    Payload(PayloadMsg),
}

#[derive(Debug, PartialEq, Eq, Clone, Encode, Decode)]
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

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ()>
        where
            Self: Sized,
    {
        Decode::decode(&mut &bytes[..]).ok_or(())
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

    pub fn as_payload(self) -> Option<Vec<u8>> {
        match self {
            Message::Payload(p) => Some(p.data),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct NetworkMessage {
    pub peer_id: AccountAddress,
    pub data: Vec<u8>,
}
