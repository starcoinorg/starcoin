// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod data_cache;
#[cfg(feature = "metrics")]
pub mod metrics;
pub mod natives;
pub mod starcoin_vm;

use move_core_types::vm_status::VMStatus;
pub use move_vm_runtime::move_vm;
pub use move_vm_runtime::session;

mod access_path_cache;
mod errors;
pub mod move_vm_ext;

use crate::metrics::VMMetrics;
use anyhow::Result;
use starcoin_vm_types::access_path::AccessPath;
use starcoin_vm_types::account_address::AccountAddress;
use starcoin_vm_types::language_storage::StructTag;
use starcoin_vm_types::state_view::StateView;
use starcoin_vm_types::transaction::{SignedUserTransaction, Transaction, TransactionOutput};

/// This trait describes the VM's validation interfaces.
pub trait VMValidator {
    /// Executes the prologue of the Aptos Account and verifies that the transaction is valid.
    fn validate_transaction(
        &mut self,
        transaction: SignedUserTransaction,
        state_view: &impl StateView,
    ) -> Option<VMStatus>;
}

/// This trait describes the VM's execution interface.
pub trait VMExecutor: Send + Sync {
    // NOTE: At the moment there are no persistent caches that live past the end of a block (that's
    // why execute_block doesn't take &self.)
    // There are some cache invalidation issues around transactions publishing code that need to be
    // sorted out before that's possible.

    /// Executes a block of transactions and returns output for each one of them.
    // fn execute_block(
    //     transactions: Vec<Transaction>,
    //     state_view: &impl StateView,
    // ) -> Result<Vec<TransactionOutput>, VMStatus>;
    fn execute_block<S: StateView>(
        chain_state: &S,
        txns: Vec<Transaction>,
        block_gas_limit: u64,
        metrics: Option<VMMetrics>,
    ) -> Result<Vec<TransactionOutput>>;
}

/// Get the AccessPath to a resource stored under `address` with type name `tag`
fn create_access_path(address: AccountAddress, tag: StructTag) -> AccessPath {
    AccessPath::resource_access_path(address, tag)
}
