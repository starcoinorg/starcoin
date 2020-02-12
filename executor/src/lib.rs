// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use config::VMConfig;
use state_store::StateStore;
use types::{
    transaction::{SignedUserTransaction, Transaction, TransactionOutput},
    vm_error::VMStatus,
};

mod mock_executor;

pub trait TransactionExecutor {
    /// Execute transaction, update state to state_store, and return events and TransactionStatus.
    fn execute_transaction(
        config: &VMConfig,
        state_store: &dyn StateStore,
        txn: Transaction,
    ) -> Result<TransactionOutput>;

    /// Executes the prologue and verifies that the transaction is valid.
    fn validate_transaction(
        config: &VMConfig,
        state_store: &dyn StateStore,
        txn: SignedUserTransaction,
    ) -> Result<VMStatus>;
}
