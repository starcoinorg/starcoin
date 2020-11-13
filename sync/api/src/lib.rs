// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use actix::prelude::*;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use starcoin_service_registry::ServiceRequest;
use starcoin_types::block::Block;
use starcoin_types::peer_info::PeerId;
use starcoin_types::sync_status::SyncStatus;

mod service;
pub use service::{SyncAsyncService, SyncServiceHandler};
pub use stream_task::TaskProgressReport;

#[derive(Message, Clone, Debug)]
#[rtype(result = "()")]
pub struct StartSyncTxnEvent;

#[derive(Message, Clone, Debug)]
#[rtype(result = "()")]
pub struct PeerNewBlock {
    peer_id: PeerId,
    new_block: Block,
}

impl PeerNewBlock {
    pub fn new(peer_id: PeerId, new_block: Block) -> Self {
        PeerNewBlock { peer_id, new_block }
    }

    pub fn get_peer_id(&self) -> PeerId {
        self.peer_id.clone()
    }

    pub fn get_block(&self) -> &Block {
        &self.new_block
    }
}

#[derive(Debug, Message, Clone, Serialize, Deserialize)]
#[rtype(result = "()")]
pub enum SyncNotify {
    ClosePeerMsg(PeerId),
    NewHeadBlock(PeerId, Box<Block>),
    NewPeerMsg(PeerId),
}

#[derive(Debug, Clone)]
pub struct SyncStatusRequest;

impl ServiceRequest for SyncStatusRequest {
    type Response = SyncStatus;
}

#[derive(Debug, Clone)]
pub struct SyncProgressRequest;

impl ServiceRequest for SyncProgressRequest {
    type Response = Option<TaskProgressReport>;
}

#[derive(Debug, Clone)]
pub struct SyncCancelRequest;

impl ServiceRequest for SyncCancelRequest {
    type Response = ();
}

#[derive(Debug, Clone)]
pub struct SyncStartRequest {
    pub force: bool,
}

impl ServiceRequest for SyncStartRequest {
    type Response = Result<()>;
}
