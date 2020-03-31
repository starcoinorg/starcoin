// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::sync_messages::*;
use crate::transaction::SignedUserTransaction;
use crate::{block::Block, peer_info::PeerId};
use actix::prelude::*;
use anyhow::*;
use futures::channel::mpsc::Sender;
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;

#[derive(Message)]
#[rtype(result = "u64")]
pub struct GetCounterMessage {}

/// message from peer
#[rtype(result = "Result<()>")]
#[derive(Debug, Serialize, Deserialize, Message)]
pub enum PeerMessage {
    UserTransactions(Vec<SignedUserTransaction>),
    Block(Block),
    LatestStateMsg(LatestStateMsg),
    RPCRequest(u128, RPCRequest),
    RPCResponse(u128, RPCResponse),
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
    GetHashByNumberMsg(ProcessMessage),
    GetDataByHashMsg(ProcessMessage),
}

#[rtype(result = "Result<()>")]
#[derive(Debug, Message, Clone)]
pub struct RpcRequestMessage {
    pub request: RPCRequest,
    pub responder: Sender<RPCResponse>,
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
    BatchHeaderAndBodyMsg(BatchHeaderMsg, BatchBodyMsg),
}

#[rtype(result = "Result<()>")]
#[derive(Debug, Eq, PartialEq, Message, Clone)]
pub enum PeerEvent {
    Open(PeerId),
    Close(PeerId),
}
