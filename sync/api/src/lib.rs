// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::cmp::Ordering;

use anyhow::Result;
use network_api::PeerId;
use network_api::PeerStrategy;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
pub use service::{SyncAsyncService, SyncServiceHandler};
use starcoin_crypto::HashValue;
use starcoin_service_registry::ServiceRequest;
use starcoin_types::block::{Block, BlockIdAndNumber, BlockInfo, BlockNumber};
use starcoin_types::sync_status::SyncStatus;
use starcoin_types::U256;
pub use stream_task::TaskProgressReport;

mod service;

#[derive(Clone, Debug, Eq)]
pub struct SyncBlockSort {
    pub block: Block,
}

impl PartialEq for SyncBlockSort {
    fn eq(&self, other: &Self) -> bool {
        self.block.header().id() == other.block.header().id()
    }
}

impl PartialOrd for SyncBlockSort {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SyncBlockSort {
    fn cmp(&self, other: &Self) -> Ordering {
        let result = self
            .block
            .header()
            .number()
            .cmp(&other.block.header().number());
        if Ordering::Equal == result {
            self.block.header().id().cmp(&other.block.header().id())
        } else {
            result
        }
    }
}

#[derive(Clone, Debug)]
pub struct StartSyncTxnEvent;

#[derive(Clone, Debug)]
pub struct PeerNewBlock {
    peer_id: PeerId,
    new_block: Block,
}

impl PeerNewBlock {
    pub fn new(peer_id: PeerId, new_block: Block) -> Self {
        Self { peer_id, new_block }
    }

    pub fn get_peer_id(&self) -> PeerId {
        self.peer_id.clone()
    }

    pub fn get_block(&self) -> &Block {
        &self.new_block
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncNotify {
    ClosePeerMsg(PeerId),
    NewHeadBlock(PeerId, Box<Block>),
    NewPeerMsg(PeerId),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncTarget {
    pub target_id: BlockIdAndNumber,
    pub block_info: BlockInfo,
    pub peers: Vec<PeerId>,
}

#[derive(Debug, Clone)]
pub struct SyncStatusRequest;

impl ServiceRequest for SyncStatusRequest {
    type Response = SyncStatus;
}

#[derive(Debug, Clone)]
pub struct SyncSpecificTargretRequest {
    pub block: Option<Block>,
    pub block_id: HashValue,
    pub peer_id: Option<PeerId>,
}

impl ServiceRequest for SyncSpecificTargretRequest {
    type Response = Result<()>;
}

#[derive(Debug, Clone)]
pub struct SyncProgressRequest;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SyncProgressReport {
    pub target_id: HashValue,
    pub begin_number: Option<BlockNumber>,
    pub target_number: BlockNumber,
    #[schemars(with = "String")]
    pub target_difficulty: U256,
    pub target_peers: Vec<PeerId>,
    pub current: TaskProgressReport,
}

impl ServiceRequest for SyncProgressRequest {
    type Response = Option<SyncProgressReport>;
}

#[derive(Debug, Clone)]
pub struct SyncCancelRequest;

impl ServiceRequest for SyncCancelRequest {
    type Response = ();
}

#[derive(Debug, Clone)]
pub struct SyncStartRequest {
    pub force: bool,
    pub peers: Vec<PeerId>,
    pub skip_pow_verify: bool,
    pub strategy: Option<PeerStrategy>,
}

impl ServiceRequest for SyncStartRequest {
    type Response = Result<()>;
}

#[derive(Debug, Clone)]
pub struct PeerScoreRequest;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PeerScoreResponse {
    peers: Option<Vec<(PeerId, u64)>>,
}

impl ServiceRequest for PeerScoreRequest {
    type Response = PeerScoreResponse;
}

impl From<Option<Vec<(PeerId, u64)>>> for PeerScoreResponse {
    fn from(peers: Option<Vec<(PeerId, u64)>>) -> Self {
        Self { peers }
    }
}
