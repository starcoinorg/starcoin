use starcoin_types2::{
    identifier::Identifier,
    language_storage::{ModuleId, TypeTag},
    vm_error::VMStatus,
};
use starcoin_vm2::{metrics::VMMetrics, starcoin_vm::StarcoinVM};
use starcoin_vm2_types::genesis_config::ChainId;
use starcoin_vm2_types::{
    transaction::{SignedUserTransaction, Transaction, TransactionOutput},
    StateView,
};

pub fn execute_transactions<S: StateView>(
    chain_state: &S,
    txns: Vec<Transaction>,
    metrics: Option<VMMetrics>,
) -> anyhow::Result<Vec<TransactionOutput>> {
    do_execute_block_transactions(chain_state, txns, None, metrics, None)
}

pub fn execute_block_transactions_with_chain_id<S: StateView>(
    chain_state: &S,
    txns: Vec<Transaction>,
    block_gas_limit: u64,
    metrics: Option<VMMetrics>,
    chain_id: &ChainId,
) -> anyhow::Result<Vec<TransactionOutput>> {
    do_execute_block_transactions(
        chain_state,
        txns,
        Some(block_gas_limit),
        metrics,
        Some(chain_id.id()),
    )
}

pub fn execute_block_transactions<S: StateView>(
    chain_state: &S,
    txns: Vec<Transaction>,
    block_gas_limit: u64,
    metrics: Option<VMMetrics>,
) -> anyhow::Result<Vec<TransactionOutput>> {
    do_execute_block_transactions(chain_state, txns, Some(block_gas_limit), metrics, None)
}

fn do_execute_block_transactions<S: StateView>(
    chain_state: &S,
    txns: Vec<Transaction>,
    block_gas_limit: Option<u64>,
    metrics: Option<VMMetrics>,
    chain_id: Option<u8>,
) -> anyhow::Result<Vec<TransactionOutput>> {
    // TODO(Bob): To determine chain id while execute genesis transaction
    let mut vm = StarcoinVM::new_with_config(metrics, chain_state, chain_id);
    let output = vm.execute_block_transactions(chain_state, txns, block_gas_limit)?;

    Ok(output.into_iter().map(|r| r.1).collect())
}

// XXX FIXME YSG, refactor use VMValidator
pub fn validate_transaction<S: StateView>(
    chain_state: &S,
    txn: SignedUserTransaction,
    metrics: Option<VMMetrics>,
) -> Option<VMStatus> {
    let mut vm = StarcoinVM::new(metrics, chain_state);
    vm.verify_transaction(chain_state, txn)
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
