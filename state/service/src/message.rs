// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use actix::Message;
use anyhow::Result;
use starcoin_crypto::HashValue;
use starcoin_state_api::StateWithProof;
use starcoin_types::{
    access_path::AccessPath, account_address::AccountAddress, account_state::AccountState,
};

#[derive(Debug, Clone)]
pub enum StateRequest {
    Get(AccessPath),
    GetWithProof(AccessPath),
    GetWithProofByRoot(AccessPath, HashValue),
    GetAccountState(AccountAddress),
    GetAccountStateByRoot(AccountAddress, HashValue),
    StateRoot(),
}

impl Message for StateRequest {
    type Result = Result<StateResponse>;
}

#[derive(Debug, Clone)]
pub enum StateResponse {
    State(Option<Vec<u8>>),
    StateWithProof(Box<StateWithProof>),
    StateRoot(HashValue),
    AccountState(Option<AccountState>),
    None,
}
