// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use crypto::HashValue;
use starcoin_config::ChainConfig;
use starcoin_state_api::ChainState;
use types::{
    account_address::AccountAddress,
    contract_event::ContractEvent,
    state_set::ChainStateSet,
    transaction::{RawUserTransaction, SignedUserTransaction, Transaction, TransactionOutput},
    vm_error::VMStatus,
};

pub mod block_executor;
pub mod executor;
#[cfg(test)]
pub mod executor_test;

pub trait TransactionExecutor: std::marker::Unpin + Clone {
    /// Create genesis state, return state root and state set.
    fn init_genesis(config: &ChainConfig)
        -> Result<(HashValue, ChainStateSet, Vec<ContractEvent>)>;

    /// Execute transactions, update state to state_store, and return State roots and TransactionOutputs.
    fn execute_transactions(
        chain_state: &dyn ChainState,
        txns: Vec<Transaction>,
    ) -> Result<Vec<(HashValue, TransactionOutput)>>;

    /// Executes the prologue and verifies that the transaction is valid.
    fn validate_transaction(
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
        receiver: AccountAddress,
        receiver_auth_key_prefix: Vec<u8>,
        seq_num: u64,
        amount: u64,
    ) -> RawUserTransaction;
}
