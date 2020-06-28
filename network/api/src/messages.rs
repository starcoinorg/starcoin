// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use actix::prelude::*;
use anyhow::*;
use futures::channel::mpsc::Sender;
use serde::{Deserialize, Serialize};
use starcoin_types::peer_info::PeerId;
use starcoin_types::peer_info::PeerInfo;
use starcoin_types::{block::BlockDetail, cmpact_block::CompactBlock, U256};

use std::borrow::Cow;
use std::sync::Arc;

#[derive(Message)]
#[rtype(result = "u64")]
pub struct GetCounterMessage {}

/// message from peer
#[rtype(result = "Result<()>")]
#[derive(Debug, Serialize, Deserialize, Message)]
#[allow(clippy::large_enum_variant)]
pub enum PeerMessage {
    Block(Arc<BlockDetail>),
    CompactBlock(CompactBlock, U256),
    RawRPCRequest(u128, String, Vec<u8>),
    RawRPCResponse(u128, Vec<u8>),
}

#[rtype(result = "Result<()>")]
#[derive(Debug, Message, Clone)]
pub struct RawRpcRequestMessage {
    pub request: Vec<u8>,
    pub responder: Sender<(Cow<'static, [u8]>, Vec<u8>)>,
}

#[rtype(result = "Result<()>")]
#[derive(Debug, Eq, PartialEq, Message, Clone)]
pub enum PeerEvent {
    Open(PeerId, Box<PeerInfo>),
    Close(PeerId),
}
