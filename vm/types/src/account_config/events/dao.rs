use anyhow::Result;
use move_core_types::account_address::AccountAddress;
use move_core_types::move_resource::MoveResource;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct ProposalCreatedEvent {
    pub proposal_id: u64,
    pub proposer: AccountAddress,
}
impl ProposalCreatedEvent {
    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        scs::from_bytes(bytes).map_err(Into::into)
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
    pub proposer: AccountAddress,
    pub voter: AccountAddress,
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
        scs::from_bytes(bytes).map_err(Into::into)
    }
}
