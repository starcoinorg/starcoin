// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    block::{Block, BlockDetail},
    cmpact_block::CompactBlock,
    peer_info::PeerId,
};
use actix::prelude::*;
use std::sync::Arc;

//TODO this type should at another crate and avoid starcoin-types dependency actix ?.
#[derive(Clone, Debug, Message)]
#[rtype(result = "()")]
pub struct NewHeadBlock(pub Arc<BlockDetail>);

#[derive(Clone, Debug, Message)]
#[rtype(result = "()")]
pub struct MinedBlock(pub Arc<Block>);

#[derive(Clone, Debug, Message)]
#[rtype(result = "()")]
pub struct SyncBegin;

#[derive(Clone, Debug, Message)]
#[rtype(result = "()")]
pub struct SyncDone;

#[derive(Message, Clone, Debug)]
#[rtype(result = "()")]
pub struct PeerNewCmpctBlock {
    pub peer_id: PeerId,
    pub compact_block: CompactBlock,
}
