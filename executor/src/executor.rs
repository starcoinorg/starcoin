// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use starcoin_types::transaction::{SignedUserTransaction, Transaction, TransactionOutput};
use starcoin_vm_types::identifier::Identifier;
use starcoin_vm_types::language_storage::{ModuleId, TypeTag};
use starcoin_vm_types::{state_view::StateView, vm_status::VMStatus};
use vm_runtime::{metrics::TXN_EXECUTION_HISTOGRAM, starcoin_vm::StarcoinVM};

pub fn execute_transactions(
    chain_state: &dyn StateView,
    txns: Vec<Transaction>,
) -> Result<Vec<TransactionOutput>> {
    do_execute_block_transactions(chain_state, txns, None)
}

/// Execute a block transactions with gas_limit,
/// if gas is used up when executing some txn, only return the outputs of previous succeed txns.
pub fn execute_block_transactions(
    chain_state: &dyn StateView,
    txns: Vec<Transaction>,
    block_gas_limit: u64,
) -> Result<Vec<TransactionOutput>> {
    do_execute_block_transactions(chain_state, txns, Some(block_gas_limit))
}

fn do_execute_block_transactions(
    chain_state: &dyn StateView,
    txns: Vec<Transaction>,
    block_gas_limit: Option<u64>,
) -> Result<Vec<TransactionOutput>> {
    let timer = TXN_EXECUTION_HISTOGRAM
        .with_label_values(&["execute_block_transactions"])
        .start_timer();
    let mut vm = StarcoinVM::new();
    let result = vm
        .execute_block_transactions(chain_state, txns, block_gas_limit)?
        .into_iter()
        .map(|(_, output)| {
            debug! {"{:?}", output};
            output
        })
        .collect();
    timer.observe_duration();
    Ok(result)
}

pub fn validate_transaction(
    chain_state: &dyn StateView,
    txn: SignedUserTransaction,
) -> Option<VMStatus> {
    let timer = TXN_EXECUTION_HISTOGRAM
        .with_label_values(&["validate_transaction"])
        .start_timer();
    let mut vm = StarcoinVM::new();
    let result = vm.verify_transaction(chain_state, txn);
    timer.observe_duration();
    result
}

pub fn execute_readonly_function(
    chain_state: &dyn StateView,
    module: &ModuleId,
    function_name: &Identifier,
    type_params: Vec<TypeTag>,
    args: Vec<Vec<u8>>,
) -> Result<Vec<Vec<u8>>, VMStatus> {
    let timer = TXN_EXECUTION_HISTOGRAM
        .with_label_values(&["execute_readonly_function"])
        .start_timer();
    let mut vm = StarcoinVM::new();
    let result =
        vm.execute_readonly_function(chain_state, module, function_name, type_params, args);
    timer.observe_duration();
    result
}
