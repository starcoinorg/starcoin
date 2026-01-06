use starcoin_metrics::metrics::VMMetrics;
use starcoin_vm2_types::{
    identifier::Identifier,
    language_storage::{ModuleId, TypeTag},
    transaction::{SignedUserTransaction, Transaction, TransactionOutput},
    vm_error::VMStatus,
};
use starcoin_vm2_vm_runtime::starcoin_vm::StarcoinVM;

use log::debug;
use starcoin_vm2_vm_runtime::VMExecutor;
use starcoin_vm2_vm_types::StateView;
use std::time::Instant;

pub fn do_execute_block_transactions<S: StateView + Sync>(
    chain_state: &S,
    txns: Vec<Transaction>,
    block_gas_limit: Option<u64>,
    metrics: Option<VMMetrics>,
) -> anyhow::Result<Vec<TransactionOutput>> {
    eprintln!(
        "jacktest do_execute_block_transactions: txn_count={}, gas_limit={:?}",
        txns.len(), block_gas_limit
    );
    let result =
        <StarcoinVM as VMExecutor>::execute_block(txns, chain_state, block_gas_limit, metrics)?;
    eprintln!(
        "jacktest do_execute_block_transactions done: output_count={}",
        result.len()
    );
    Ok(result)
}

#[allow(dead_code)]
pub fn validate_transaction<S: StateView>(
    chain_state: &S,
    txn: SignedUserTransaction,
    metrics: Option<VMMetrics>,
) -> Option<VMStatus> {
    let total_start = Instant::now();
    let init_start = Instant::now();
    let mut vm = StarcoinVM::new(metrics, chain_state);
    let init_dur = init_start.elapsed();

    let verify_start = Instant::now();
    let result = vm.verify_transaction(chain_state, txn);
    let verify_dur = verify_start.elapsed();

    debug!(
        target: "txpool",
        "vm2 validate_transaction init_ms={:.3} verify_ms={:.3} total_ms={:.3}",
        init_dur.as_secs_f64() * 1000.0,
        verify_dur.as_secs_f64() * 1000.0,
        total_start.elapsed().as_secs_f64() * 1000.0,
    );
    result
}

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
