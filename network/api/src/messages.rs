// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use actix::prelude::*;
use anyhow::*;
use futures::channel::mpsc::Sender;
use serde::{Deserialize, Serialize};
use starcoin_types::peer_info::PeerId;
use starcoin_types::peer_info::PeerInfo;
use starcoin_types::transaction::SignedUserTransaction;
use starcoin_types::{cmpact_block::CompactBlock, U256};
use std::borrow::Cow;

#[derive(Message)]
#[rtype(result = "u64")]
pub struct GetCounterMessage {}

/// message from peer
#[rtype(result = "Result<()>")]
#[derive(Clone, Debug, Serialize, Deserialize, Message)]
#[allow(clippy::large_enum_variant)]
pub enum PeerMessage {
    NewTransactions(Vec<SignedUserTransaction>),
    CompactBlock(CompactBlock, U256),
    RawRPCRequest(u128, String, Vec<u8>),
    RawRPCResponse(u128, Vec<u8>),
}

#[rtype(result = "Result<()>")]
#[derive(Debug, Message, Clone)]
pub struct RawRpcRequestMessage {
    pub request: (String, Vec<u8>, PeerId),
    pub responder: Sender<(Cow<'static, [u8]>, Vec<u8>)>,
}

#[rtype(result = "Result<()>")]
#[derive(Debug, Eq, PartialEq, Message, Clone)]
pub enum PeerEvent {
    Open(PeerId, Box<PeerInfo>),
    Close(PeerId),
}
