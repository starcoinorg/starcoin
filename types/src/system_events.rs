// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::block::{Block, BlockDetail, BlockHeader};
use actix::prelude::*;
use std::sync::Arc;

//TODO this type should at another crate and avoid starcoin-types dependency actix ?.
#[derive(Clone, Debug, Message)]
#[rtype(result = "()")]
pub struct NewHeadBlock(pub Arc<BlockDetail>);

/// may be uncle block
#[derive(Clone, Debug, Message)]
#[rtype(result = "()")]
pub struct NewBranch(pub Arc<BlockHeader>);

#[derive(Clone, Debug, Message)]
#[rtype(result = "()")]
pub struct MinedBlock(pub Arc<Block>);

#[derive(Clone, Debug, Message)]
#[rtype(result = "()")]
pub struct SyncBegin;

#[derive(Clone, Debug, Message)]
#[rtype(result = "()")]
pub struct SyncDone;

/// Try to stop a actor
#[derive(Clone, Debug, Message)]
#[rtype(result = "()")]
pub struct ActorStop;

/// Try to stop system.
#[derive(Clone, Debug, Message)]
#[rtype(result = "()")]
pub struct SystemStop;

///Fire this event on System start and all service is init.
#[derive(Clone, Debug, Message)]
#[rtype(result = "()")]
pub struct SystemStarted;
