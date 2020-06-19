// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use starcoin_types::{
    transaction::{SignedUserTransaction, Transaction, TransactionOutput},
    vm_error::VMStatus,
};
use starcoin_vm_types::state_view::StateView;
use vm_runtime::{metrics::TXN_EXECUTION_HISTOGRAM, starcoin_vm::StarcoinVM};

pub fn execute_transactions(
    chain_state: &dyn StateView,
    txns: Vec<Transaction>,
) -> Result<Vec<TransactionOutput>> {
    let timer = TXN_EXECUTION_HISTOGRAM
        .with_label_values(&["execute_transactions"])
        .start_timer();
    let mut vm = StarcoinVM::new();
    let result = vm.execute_transactions(chain_state, txns)?;
    timer.observe_duration();
    Ok(result)
}

/// Execute a block transactions with gas_limit,
/// if gas is used up when executing some txn, only return the outputs of previous succeed txns.
pub fn execute_block_transactions(
    chain_state: &dyn StateView,
    txns: Vec<Transaction>,
    block_gas_limit: u64,
) -> Result<Vec<TransactionOutput>> {
    let timer = TXN_EXECUTION_HISTOGRAM
        .with_label_values(&["execute_block_transactions"])
        .start_timer();
    let mut vm = StarcoinVM::new();
    let result = vm.execute_block_transactions(chain_state, txns, Some(block_gas_limit))?;
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
