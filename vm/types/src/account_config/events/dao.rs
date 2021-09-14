// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::move_resource::MoveResource;
use anyhow::Result;
use move_core_types::account_address::AccountAddress;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct ProposalCreatedEvent {
    pub proposal_id: u64,
    pub proposer: AccountAddress,
}
impl ProposalCreatedEvent {
    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        bcs_ext::from_bytes(bytes).map_err(Into::into)
    }
}
impl MoveResource for ProposalCreatedEvent {
    const MODULE_NAME: &'static str = "Dao";
    const STRUCT_NAME: &'static str = "ProposalCreatedEvent";
}

/// emitted when user vote/revoke_vote.
#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct VoteChangedEvent {
    pub proposal_id: u64,
    pub voter: AccountAddress,
    pub proposer: AccountAddress,
    pub agree: bool,
    /// latest vote of the voter.
    pub vote: u128,
}

impl MoveResource for VoteChangedEvent {
    const MODULE_NAME: &'static str = "Dao";
    const STRUCT_NAME: &'static str = "VoteChangedEvent";
}

impl VoteChangedEvent {
    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        bcs_ext::from_bytes(bytes).map_err(Into::into)
    }
}
