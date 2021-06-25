// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::access_path::AccessPath;
use crate::account_address::AccountAddress;
use crate::account_config::CORE_CODE_ADDRESS;
use crate::event::EventHandle;
use crate::language_storage::{StructTag, TypeTag};
use crate::move_resource::MoveResource;
use serde::{Deserialize, Serialize};

mod actions;
pub use actions::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct DaoGlobalInfo {
    /// next proposal id.
    pub next_proposal_id: u64,
    /// proposal creating event.
    pub proposal_create_event: EventHandle,
    /// voting event.
    pub vote_changed_event: EventHandle,
}

impl MoveResource for DaoGlobalInfo {
    const MODULE_NAME: &'static str = "Dao";
    const STRUCT_NAME: &'static str = "DaoGlobalInfo";
}

impl DaoGlobalInfo {
    pub fn struct_tag_for(token_type_tag: StructTag) -> StructTag {
        StructTag {
            address: CORE_CODE_ADDRESS,
            module: DaoGlobalInfo::module_identifier(),
            name: DaoGlobalInfo::struct_identifier(),
            type_params: vec![TypeTag::Struct(token_type_tag)],
        }
    }

    pub fn resource_path_for(token_type_tag: StructTag) -> AccessPath {
        AccessPath::resource_access_path(
            token_type_tag.address,
            DaoGlobalInfo::struct_tag_for(token_type_tag),
        )
    }
}

//TODO should add Serialize and Deserialize require to MoveResource trait?
pub trait ProposalAction: MoveResource + Serialize {}

#[derive(Debug, Serialize, Deserialize)]
pub struct Proposal<A: ProposalAction> {
    /// id of the proposal
    pub id: u64,
    /// creator of the proposal
    pub proposer: AccountAddress,
    /// when voting begins.
    pub start_time: u64,
    /// when voting ends.
    pub end_time: u64,
    /// count of votes for agree.
    pub for_votes: u128,
    /// count of votes for againest.
    pub against_votes: u128,
    /// executable after this time.
    pub eta: u64,
    /// after how long, the agreed proposal can be executed.
    pub action_delay: u64,
    /// how many votes to reach to make the proposal pass.
    pub quorum_votes: u128,
    /// proposal action.
    pub action: Option<A>,
}

impl<A> MoveResource for Proposal<A>
where
    A: ProposalAction,
{
    const MODULE_NAME: &'static str = "Dao";
    const STRUCT_NAME: &'static str = "Proposal";
}

impl<A> Proposal<A>
where
    A: ProposalAction,
{
    pub fn struct_tag_for(token_type_tag: StructTag) -> StructTag {
        StructTag {
            address: CORE_CODE_ADDRESS,
            module: Self::module_identifier(),
            name: Self::struct_identifier(),
            type_params: vec![TypeTag::Struct(token_type_tag), A::type_tag()],
        }
    }

    pub fn resource_path_for(token_type_tag: StructTag) -> AccessPath {
        AccessPath::resource_access_path(
            token_type_tag.address,
            Self::struct_tag_for(token_type_tag),
        )
    }
}

/// User vote info.
#[derive(Debug, Serialize, Deserialize)]
pub struct Vote {
    /// vote for the proposal under the `proposer`.
    pub proposer: AccountAddress,
    /// proposal id.
    pub id: u64,
    /// how many tokens to stake.
    pub stake: u128,
    /// vote for or vote against.
    pub agree: bool,
}

impl MoveResource for Vote {
    const MODULE_NAME: &'static str = "Dao";
    const STRUCT_NAME: &'static str = "Vote";
}
