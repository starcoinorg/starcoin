// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use actix::prelude::*;
use anyhow::*;
use futures::channel::mpsc::Sender;
use serde::{Deserialize, Serialize};
use starcoin_service_registry::ServiceRequest;
use starcoin_types::peer_info::PeerId;
use starcoin_types::peer_info::PeerInfo;
use starcoin_types::transaction::SignedUserTransaction;
use starcoin_types::{cmpact_block::CompactBlock, U256};

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
    pub request: (PeerId, String, Vec<u8>),
    pub responder: Sender<Vec<u8>>,
}

// TODO remove RawRpcRequestMessage responder and set response.
impl ServiceRequest for RawRpcRequestMessage {
    type Response = ();
}

#[rtype(result = "Result<()>")]
#[derive(Debug, Eq, PartialEq, Message, Clone)]
pub enum PeerEvent {
    Open(PeerId, Box<PeerInfo>),
    Close(PeerId),
}
