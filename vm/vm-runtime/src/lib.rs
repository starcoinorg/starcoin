// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_config::config::VMConfig;
use starcoin_state_view::StateView;
use starcoin_types::{
    transaction::{SignedTransaction, Transaction, TransactionOutput},
    vm_error::VMStatus,
};

/// This trait describes the VM's verification interfaces.
pub trait VMVerifier {
    /// Executes the prologue of the Libra Account and verifies that the transaction is valid.
    /// only. Returns `None` if the transaction was validated, or Some(VMStatus) if the transaction
    /// was unable to be validated with status `VMStatus`.
    fn validate_transaction(
        &self,
        transaction: SignedTransaction,
        state_view: &dyn StateView,
    ) -> Option<VMStatus>;
}

/// This trait describes the VM's execution interface.
pub trait VMExecutor {
    /// Executes a block of transactions and returns output for each one of them.
    fn execute_block(
        transactions: Vec<Transaction>,
        config: &VMConfig,
        state_view: &dyn StateView,
    ) -> Result<Vec<TransactionOutput>, VMStatus>;
}
