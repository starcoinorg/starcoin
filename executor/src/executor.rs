// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::TransactionExecutor;
use anyhow::Result;
use starcoin_config::ChainConfig;
use starcoin_types::{
    account_address::AccountAddress,
    transaction::{RawUserTransaction, SignedUserTransaction, Transaction, TransactionOutput},
    vm_error::VMStatus,
};
use starcoin_vm_types::{state_view::StateView, transaction::ChangeSet};
use vm_runtime::genesis::generate_genesis_state_set;
use vm_runtime::{
    common_transactions::{peer_to_peer_txn_sent_as_association, raw_peer_to_peer_txn},
    counters::TXN_EXECUTION_HISTOGRAM,
    starcoin_vm::StarcoinVM,
};

#[derive(Clone, Default)]
pub struct Executor {}

impl Executor {
    /// Creates an executor in which no genesis state has been applied yet.
    pub fn new() -> Self {
        Executor {}
    }
}

impl TransactionExecutor for Executor {
    fn init_genesis(chain_config: &ChainConfig) -> Result<ChangeSet> {
        let timer = TXN_EXECUTION_HISTOGRAM
            .with_label_values(&["init_genesis"])
            .start_timer();

        let change_set = generate_genesis_state_set(&chain_config)?;

        timer.observe_duration();
        Ok(change_set)
    }

    fn execute_transactions(
        chain_state: &dyn StateView,
        txns: Vec<Transaction>,
    ) -> Result<Vec<TransactionOutput>> {
        let timer = TXN_EXECUTION_HISTOGRAM
            .with_label_values(&["execute_transactions"])
            .start_timer();
        let mut vm = StarcoinVM::new();
        let result = vm.execute_transactions(chain_state, txns)?;
        timer.observe_duration();
        Ok(result)
    }

    /// Execute a block transactions with gas_limit,
    /// if gas is used up when executing some txn, only return the outputs of previous succeed txns.
    fn execute_block_transactions(
        chain_state: &dyn StateView,
        txns: Vec<Transaction>,
        block_gas_limit: u64,
    ) -> Result<Vec<TransactionOutput>> {
        let timer = TXN_EXECUTION_HISTOGRAM
            .with_label_values(&["execute_block_transactions"])
            .start_timer();
        let mut vm = StarcoinVM::new();
        let result = vm.execute_block_transactions(chain_state, txns, Some(block_gas_limit))?;
        timer.observe_duration();
        Ok(result)
    }

    fn validate_transaction(
        chain_state: &dyn StateView,
        txn: SignedUserTransaction,
    ) -> Option<VMStatus> {
        let timer = TXN_EXECUTION_HISTOGRAM
            .with_label_values(&["validate_transaction"])
            .start_timer();
        let mut vm = StarcoinVM::new();
        let result = vm.verify_transaction(chain_state, txn);
        timer.observe_duration();
        result
    }

    fn build_mint_txn(
        addr: AccountAddress,
        auth_key_prefix: Vec<u8>,
        seq_num: u64,
        amount: u64,
    ) -> Transaction {
        Transaction::UserTransaction(peer_to_peer_txn_sent_as_association(
            addr,
            auth_key_prefix,
            seq_num,
            amount,
        ))
    }

    fn build_transfer_txn(
        sender: AccountAddress,
        receiver: AccountAddress,
        receiver_auth_key_prefix: Vec<u8>,
        seq_num: u64,
        amount: u64,
        gas_price: u64,
        max_gas: u64,
    ) -> RawUserTransaction {
        raw_peer_to_peer_txn(
            sender,
            receiver,
            receiver_auth_key_prefix,
            amount,
            seq_num,
            gas_price,
            max_gas,
        )
    }
}
