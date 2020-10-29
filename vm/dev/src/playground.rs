// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use starcoin_crypto::HashValue;
use starcoin_resource_viewer::{AnnotatedMoveValue, MoveValueAnnotator};
use starcoin_state_api::StateNodeStore;
use starcoin_statedb::ChainStateDB;
use starcoin_vm_runtime::starcoin_vm::{convert_txn_args, StarcoinVM};
use starcoin_vm_types::account_address::AccountAddress;
use starcoin_vm_types::identifier::{IdentStr, Identifier};
use starcoin_vm_types::language_storage::{ModuleId, TypeTag};
use starcoin_vm_types::state_view::StateView;
use starcoin_vm_types::transaction::{Transaction, TransactionOutput};
use starcoin_vm_types::transaction_argument::TransactionArgument;
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

    pub fn call_contract(
        &self,
        state_root: HashValue,
        module_address: AccountAddress,
        module_name: String,
        func: String,
        type_args: Vec<TypeTag>,
        args: Vec<TransactionArgument>,
    ) -> Result<Vec<AnnotatedMoveValue>> {
        let state_view = ChainStateDB::new(self.state.clone(), Some(state_root));
        let module_id = ModuleId::new(module_address, Identifier::new(module_name)?);
        let rets = call_contract(&state_view, module_id, func.as_str(), type_args, args)?;
        Ok(rets)
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

pub fn call_contract(
    state_view: &dyn StateView,
    module_id: ModuleId,
    func: &str,
    type_args: Vec<TypeTag>,
    args: Vec<TransactionArgument>,
) -> Result<Vec<AnnotatedMoveValue>> {
    let mut vm = StarcoinVM::new();
    let rets = vm.execute_readonly_function(
        state_view,
        &module_id,
        &IdentStr::new(func)?,
        type_args,
        convert_txn_args(&args),
    )?;
    let annotator = MoveValueAnnotator::new(state_view);
    let mut annotated_values = Vec::with_capacity(rets.len());
    for (t, v) in rets {
        let layout = annotator.type_tag_to_type_layout(&t)?;
        let r = v
            .simple_serialize(&layout)
            .ok_or_else(|| anyhow::format_err!("fail to serialize contract result"))?;
        let value = annotator.view_value(&t, r.as_slice())?;
        annotated_values.push(value);
    }
    Ok(annotated_values)
}
