// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod adapter_common;
pub mod data_cache;
#[cfg(feature = "metrics")]
pub mod metrics;
pub mod natives;
pub mod starcoin_vm;

use move_core_types::vm_status::VMStatus;
pub use move_vm_runtime::{move_vm, session};
mod access_path_cache;
mod errors;
pub mod force_upgrade_management;
pub mod move_vm_ext;
pub mod parallel_executor;
use crate::metrics::VMMetrics;
use starcoin_vm_types::{
    access_path::AccessPath,
    account_address::AccountAddress,
    language_storage::StructTag,
    state_view::StateView,
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
