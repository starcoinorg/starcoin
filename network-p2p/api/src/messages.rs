// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use actix::prelude::*;
use anyhow::*;
use futures::channel::mpsc::Sender;
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;
use starcoin_state_tree::StateNode;
use starcoin_sync_api::sync_messages::*;
use starcoin_types::block::BlockDetail;
use starcoin_types::peer_info::PeerId;
use starcoin_types::peer_info::PeerInfo;
use starcoin_types::transaction::SignedUserTransaction;

#[derive(Message)]
#[rtype(result = "u64")]
pub struct GetCounterMessage {}

/// message from peer
#[rtype(result = "Result<()>")]
#[derive(Debug, Serialize, Deserialize, Message)]
pub enum PeerMessage {
    UserTransactions(Vec<SignedUserTransaction>),
    Block(BlockDetail),
    RPCRequest(u128, RPCRequest),
    RPCResponse(u128, RPCResponse),
    RawRPCRequest(u128, Vec<u8>),
    RawRPCResponse(u128, Vec<u8>),
}

#[rtype(result = "Result<()>")]
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Message, Clone)]
pub struct TestRequest {
    pub data: HashValue,
}

/// message from peer
#[rtype(result = "Result<()>")]
#[derive(Debug, Serialize, Deserialize, Message, Clone)]
pub enum RPCRequest {
    TestRequest(TestRequest),
    GetStateNodeByNodeHash(HashValue),
}

#[rtype(result = "Result<()>")]
#[derive(Debug, Message, Clone)]
pub struct RpcRequestMessage {
    pub request: RPCRequest,
    pub responder: Sender<RPCResponse>,
}

#[rtype(result = "Result<()>")]
#[derive(Debug, Message, Clone)]
pub struct RawRpcRequestMessage {
    pub request: Vec<u8>,
    pub responder: Sender<Vec<u8>>,
}

#[rtype(result = "Result<()>")]
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Message, Clone)]
pub struct TestResponse {
    pub len: u8,
    pub id: HashValue,
}

#[rtype(result = "Result<()>")]
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Message, Clone)]
pub enum RPCResponse {
    TestResponse(TestResponse),
    BatchHashByNumberMsg(BatchHashByNumberMsg),
    BatchHeaderAndBodyMsg(BatchHeaderMsg, BatchBodyMsg, BatchBlockInfo),
    GetStateNodeByNodeHash(StateNode),
}

#[rtype(result = "Result<()>")]
#[derive(Debug, Eq, PartialEq, Message, Clone)]
pub enum PeerEvent {
    Open(PeerId, PeerInfo),
    Close(PeerId),
}
