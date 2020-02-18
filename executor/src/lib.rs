// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use chain_state::ChainState;
use config::VMConfig;
use types::{
    transaction::{SignedUserTransaction, Transaction, TransactionOutput},
    vm_error::VMStatus,
};

mod executor_test;
pub mod mock_executor;

pub trait TransactionExecutor {
    /// Execute transaction, update state to state_store, and return events and TransactionStatus.
    fn execute_transaction(
        config: &VMConfig,
        chain_state: &dyn ChainState,
        txn: Transaction,
    ) -> Result<TransactionOutput>;

    /// Executes the prologue and verifies that the transaction is valid.
    fn validate_transaction(
        config: &VMConfig,
        chain_state: &dyn ChainState,
        txn: SignedUserTransaction,
    ) -> Result<VMStatus>;
}
