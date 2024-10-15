// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

//pub mod data_cache;
#[cfg(feature = "metrics")]
//pub mod natives;
pub mod starcoin_vm;

//#[macro_use]
//pub mod counters;

//use move_core_types::vm_status::VMStatus;
//pub use move_vm_runtime::{move_vm, session};
//mod access_path_cache;
//mod errors;
//pub mod move_vm_ext;
//pub mod parallel_executor;
//mod verifier;
//mod vm_adapter;

use move_core_types::vm_status::VMStatus;
pub use starcoin_metrics::metrics::VMMetrics;
use starcoin_vm_types::block_metadata::BlockMetadata;
use starcoin_vm_types::state_store::StateView;
use starcoin_vm_types::transaction::SignedUserTransaction;
use starcoin_vm_types::{
    access_path::AccessPath,
    account_address::AccountAddress,
    language_storage::StructTag,
    transaction::{Transaction, TransactionOutput},
};

/// This trait describes the VM's execution interface.
pub trait VMExecutor: Send + Sync {
    // NOTE: At the moment there are no persistent caches that live past the end of a block (that's
    // why execute_block doesn't take &self.)
    // There are some cache invalidation issues around transactions publishing code that need to be
    // sorted out before that's possible.

    /// Executes a block of transactions and returns output for each one of them.
    fn execute_block(
        transactions: Vec<Transaction>,
        state_view: &impl StateView,
        block_gas_limit: Option<u64>,
        metrics: Option<VMMetrics>,
    ) -> Result<Vec<TransactionOutput>, VMStatus>;
}
/// Get the AccessPath to a resource stored under `address` with type name `tag`
fn create_access_path(address: AccountAddress, tag: StructTag) -> AccessPath {
    AccessPath::resource_access_path(address, tag)
}

#[derive(Debug)]
pub enum PreprocessedTransaction {
    UserTransaction(Box<SignedUserTransaction>),
    BlockMetadata(BlockMetadata),
}

pub trait VMAdapter {
    /// TODO: maybe remove this after more refactoring of execution logic.
    fn should_restart_execution(output: &TransactionOutput) -> bool;

    /// Execute a single transaction.
    fn execute_single_transaction<S: StateView>(
        &self,
        txn: &PreprocessedTransaction,
        data_cache: &S,
    ) -> anyhow::Result<(VMStatus, TransactionOutput, Option<String>), VMStatus>;
}
