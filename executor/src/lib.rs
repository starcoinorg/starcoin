// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use config::VMConfig;
use crypto::HashValue;
use starcoin_state_api::ChainState;
use types::{
    account_address::AccountAddress,
    state_set::ChainStateSet,
    transaction::{RawUserTransaction, SignedUserTransaction, Transaction, TransactionOutput},
    vm_error::VMStatus,
};

pub mod executor;
pub mod executor_test;
pub mod mock_executor;

pub trait TransactionExecutor: std::marker::Unpin + Clone {
    /// Create genesis state, return state root and state set.
    fn init_genesis(config: &VMConfig) -> Result<(HashValue, ChainStateSet)>;

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
    ) -> Option<VMStatus>;

    fn build_mint_txn(
        addr: AccountAddress,
        auth_key_prefix: Vec<u8>,
        seq_num: u64,
        amount: u64,
    ) -> Transaction;

    fn build_transfer_txn(
        sender: AccountAddress,
        sender_auth_key_prefix: Vec<u8>,
        receiver: AccountAddress,
        receiver_auth_key_prefix: Vec<u8>,
        seq_num: u64,
        amount: u64,
    ) -> RawUserTransaction;
}
