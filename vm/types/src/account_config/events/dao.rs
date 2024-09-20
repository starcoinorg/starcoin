// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use move_core_types::account_address::AccountAddress;
use move_core_types::ident_str;
use move_core_types::identifier::IdentStr;
use move_core_types::move_resource::{MoveResource, MoveStructType};
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
impl MoveStructType for ProposalCreatedEvent {
    const MODULE_NAME: &'static IdentStr = ident_str!("Dao");
    const STRUCT_NAME: &'static IdentStr = ident_str!("ProposalCreatedEvent");
}

impl MoveResource for ProposalCreatedEvent {}

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

impl MoveStructType for VoteChangedEvent {
    const MODULE_NAME: &'static IdentStr = ident_str!("Dao");
    const STRUCT_NAME: &'static IdentStr = ident_str!("VoteChangedEvent");
}

impl MoveResource for VoteChangedEvent {}

impl VoteChangedEvent {
    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        bcs_ext::from_bytes(bytes).map_err(Into::into)
    }
}
