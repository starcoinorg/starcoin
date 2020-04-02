// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::TransactionExecutor;
use anyhow::Result;
use config::VMConfig;
use crypto::HashValue;
use starcoin_state_api::ChainState;
use statedb::ChainStateDB;
use std::sync::Arc;
use storage::{cache_storage::CacheStorage, db_storage::DBStorage, StarcoinStorage};
use types::{
    account_address::AccountAddress,
    state_set::ChainStateSet,
    transaction::{RawUserTransaction, SignedUserTransaction, Transaction, TransactionOutput},
    vm_error::VMStatus,
};
use vm_runtime::genesis::{generate_genesis_state_set, GENESIS_KEYPAIR};
use vm_runtime::{
    account::Account,
    common_transactions::{
        create_account_txn_sent_as_association, peer_to_peer_txn_sent_as_association,
        raw_peer_to_peer_txn,
    },
    starcoin_vm::StarcoinVM,
};

#[derive(Clone)]
pub struct Executor {}

impl Executor {
    /// Creates an executor in which no genesis state has been applied yet.
    pub fn new() -> Self {
        Executor {}
    }
}

impl TransactionExecutor for Executor {
    fn init_genesis(_config: &VMConfig) -> Result<(HashValue, ChainStateSet)> {
        let cache_storage = Arc::new(CacheStorage::new());
        let tmpdir = libra_temppath::TempPath::new();
        let db_storage = Arc::new(DBStorage::new(tmpdir.path()));
        let storage = Arc::new(StarcoinStorage::new(cache_storage, db_storage).unwrap());
        let chain_state = ChainStateDB::new(storage, None);

        // ToDo: load genesis txn from genesis.blob, instead of generating from stdlib
        generate_genesis_state_set(&GENESIS_KEYPAIR.0, GENESIS_KEYPAIR.1.clone(), &chain_state)
    }

    fn execute_transaction(
        config: &VMConfig,
        chain_state: &dyn ChainState,
        txn: Transaction,
    ) -> Result<TransactionOutput> {
        let mut vm = StarcoinVM::new(config);
        let output = vm.execute_transaction(chain_state, txn);
        Ok(output)
    }

    fn validate_transaction(
        config: &VMConfig,
        chain_state: &dyn ChainState,
        txn: SignedUserTransaction,
    ) -> Option<VMStatus> {
        let mut vm = StarcoinVM::new(config);
        vm.verify_transaction(chain_state, txn)
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
        sender_auth_key_prefix: Vec<u8>,
        receiver: AccountAddress,
        receiver_auth_key_prefix: Vec<u8>,
        seq_num: u64,
        amount: u64,
    ) -> RawUserTransaction {
        raw_peer_to_peer_txn(
            sender,
            sender_auth_key_prefix,
            receiver,
            receiver_auth_key_prefix,
            amount,
            seq_num,
        )
    }
}

pub fn mock_create_account_txn() -> Transaction {
    let account1 = Account::new();
    Transaction::UserTransaction(create_account_txn_sent_as_association(
        &account1, 1, // fix me
        1_000,
    ))
}
