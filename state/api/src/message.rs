// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::StateWithProof;
use anyhow::Result;
use starcoin_crypto::HashValue;
use starcoin_service_registry::ServiceRequest;
use starcoin_types::state_set::AccountStateSet;
use starcoin_types::{
    access_path::AccessPath, account_address::AccountAddress, account_state::AccountState,
};

#[derive(Debug, Clone)]
pub enum StateRequest {
    Get(AccessPath),
    GetWithProof(AccessPath),
    GetWithProofByRoot(AccessPath, HashValue),
    GetAccountState(AccountAddress),
    GetAccountStateSet {
        address: AccountAddress,
        state_root: Option<HashValue>,
    },
    GetAccountStateByRoot(AccountAddress, HashValue),
    StateRoot(),
}

impl ServiceRequest for StateRequest {
    type Response = Result<StateResponse>;
}

#[derive(Debug, Clone)]
pub enum StateResponse {
    State(Option<Vec<u8>>),
    StateWithProof(Box<StateWithProof>),
    StateRoot(HashValue),
    AccountState(Option<AccountState>),
    AccountStateSet(Option<AccountStateSet>),
    None,
}
