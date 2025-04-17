// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{StateWithProof, StateWithTableItemProof};
use anyhow::Result;
use starcoin_crypto::HashValue;
use starcoin_service_registry::ServiceRequest;
use starcoin_types::state_set::AccountStateSet;
use starcoin_types::{
    access_path::AccessPath, account_address::AccountAddress, account_state::AccountState,
};
use starcoin_vm_types::state_store::table::{TableHandle, TableInfo};

#[derive(Debug, Clone)]
pub enum StateRequestVMType {
    MoveVm1,
    MoveVm2,
}

#[derive(Debug, Clone)]
pub enum StateRequest {
    Get(StateRequestVMType, AccessPath),
    GetWithProof(StateRequestVMType, AccessPath),
    GetWithProofByRoot(StateRequestVMType, AccessPath, HashValue),
    GetAccountState(StateRequestVMType, AccountAddress),
    GetAccountStateSet {
        vm_type: StateRequestVMType,
        address: AccountAddress,
        state_root: Option<HashValue>,
    },
    GetAccountStateByRoot(StateRequestVMType, AccountAddress, HashValue),
    StateRoot(StateRequestVMType),
    GetWithTableItemProof(StateRequestVMType, TableHandle, Vec<u8>),
    GetWithTableItemProofByRoot(StateRequestVMType, TableHandle, Vec<u8>, HashValue),
    GetTableInfo(StateRequestVMType, AccountAddress),
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
    StateWithTableItemProof(Box<StateWithTableItemProof>),
    TableInfo(Option<TableInfo>),
}
