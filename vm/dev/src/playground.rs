// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use starcoin_crypto::HashValue;
use starcoin_state_api::StateNodeStore;
use starcoin_statedb::ChainStateDB;
use starcoin_vm_runtime::starcoin_vm::StarcoinVM;
use starcoin_vm_types::state_view::StateView;
use starcoin_vm_types::transaction::{Transaction, TransactionOutput};
use starcoin_vm_types::vm_status::VMStatus;
use std::sync::Arc;

#[derive(Clone)]
pub struct PlaygroudService {
    state: Arc<dyn StateNodeStore>,
}

impl PlaygroudService {
    pub fn new(state_store: Arc<dyn StateNodeStore>) -> Self {
        Self { state: state_store }
    }
}

impl PlaygroudService {
    pub fn dry_run(
        &self,
        state_root: HashValue,
        txn: Transaction,
    ) -> Result<(VMStatus, TransactionOutput)> {
        let state_view = ChainStateDB::new(self.state.clone(), Some(state_root));
        dry_run(&state_view, txn)
    }
}

pub fn dry_run(
    state_view: &dyn StateView,
    txn: Transaction,
) -> Result<(VMStatus, TransactionOutput)> {
    let mut vm = StarcoinVM::new();
    vm.execute_transactions(state_view, vec![txn])
        .map(|mut r| r.pop().unwrap())
}
