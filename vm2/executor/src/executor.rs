// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_vm2_types::{
    identifier::Identifier,
    language_storage::{ModuleId, TypeTag},
    transaction::{SignedUserTransaction, Transaction, TransactionOutput},
    vm_error::VMStatus,
};
use starcoin_vm2_vm_runtime::{metrics::VMMetrics, starcoin_vm::StarcoinVM};

use starcoin_vm2_vm_types::StateView;

pub fn do_execute_block_transactions<S: StateView>(
    chain_state: &S,
    txns: Vec<Transaction>,
    block_gas_limit: Option<u64>,
    metrics: Option<VMMetrics>,
) -> anyhow::Result<Vec<TransactionOutput>> {
    let mut vm = StarcoinVM::new(metrics, chain_state);
    let output = vm.execute_block_transactions(chain_state, txns, block_gas_limit)?;

    Ok(output.into_iter().map(|r| r.1).collect())
}

#[allow(dead_code)]
pub fn validate_transaction<S: StateView>(
    chain_state: &S,
    txn: SignedUserTransaction,
    metrics: Option<VMMetrics>,
) -> Option<VMStatus> {
    let mut vm = StarcoinVM::new(metrics, chain_state);
    vm.verify_transaction(chain_state, txn)
}

#[allow(dead_code)]
pub fn execute_readonly_function<S: StateView>(
    chain_state: &S,
    module: &ModuleId,
    function_name: &Identifier,
    type_params: Vec<TypeTag>,
    args: Vec<Vec<u8>>,
    metrics: Option<VMMetrics>,
) -> anyhow::Result<Vec<Vec<u8>>, VMStatus> {
    let mut vm = StarcoinVM::new(metrics, chain_state);
    vm.execute_readonly_function(chain_state, module, function_name, type_params, args)
}
