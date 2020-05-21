// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use starcoin_config::ChainConfig;
use starcoin_types::{
    account_address::AccountAddress,
    transaction::{RawUserTransaction, SignedUserTransaction, Transaction, TransactionOutput},
    vm_error::VMStatus,
};
use starcoin_vm_types::state_view::StateView;
use starcoin_vm_types::transaction::ChangeSet;

pub mod block_executor;
pub mod executor;
#[cfg(test)]
pub mod executor_test;

pub trait TransactionExecutor: std::marker::Unpin + Clone {
    /// Create genesis state, return state root and state set.
    fn init_genesis(config: &ChainConfig) -> Result<ChangeSet>;

    /// Execute transactions, update state to state_store, and return State roots and TransactionOutputs.
    fn execute_transactions(
        state_view: &dyn StateView,
        txns: Vec<Transaction>,
    ) -> Result<Vec<TransactionOutput>>;

    /// Executes the prologue and verifies that the transaction is valid.
    fn validate_transaction(
        state_view: &dyn StateView,
        txn: SignedUserTransaction,
    ) -> Option<VMStatus>;

    fn build_mint_txn(
        addr: AccountAddress,
        auth_key_prefix: Vec<u8>,
        seq_num: u64,
        amount: u64,
    ) -> Transaction;

    fn build_transfer_txn(
        sender: AccountAddress,
        receiver: AccountAddress,
        receiver_auth_key_prefix: Vec<u8>,
        seq_num: u64,
        amount: u64,
        gas_price: u64,
        max_gas: u64,
    ) -> RawUserTransaction;
}
