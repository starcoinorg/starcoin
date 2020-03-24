// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::TransactionExecutor;
use anyhow::Result;

use config::VMConfig;
use crypto::HashValue;
use statedb::ChainStateDB;
use std::sync::Arc;
use storage::cache_storage::CacheStorage;
use storage::db_storage::DBStorage;
use storage::StarcoinStorage;
use traits::ChainState;
use types::{
    state_set::ChainStateSet,
    transaction::{SignedUserTransaction, Transaction, TransactionOutput},
    vm_error::VMStatus,
};
use vm_runtime::genesis::{generate_genesis_state_set, GENESIS_KEYPAIR};
use vm_runtime::starcoin_vm::StarcoinVM;
use vm_runtime::{
    account::Account,
    common_transactions::{create_account_txn_send_with_association_account, peer_to_peer_txn},
};

pub struct Executor {
    config: VMConfig,
}

impl Executor {
    /// Creates an executor in which no genesis state has been applied yet.
    pub fn new() -> Self {
        Executor {
            config: VMConfig::default(),
        }
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
        _config: &VMConfig,
        _chain_state: &dyn ChainState,
        _txn: SignedUserTransaction,
    ) -> Option<VMStatus> {
        None
    }
}

pub fn mock_create_account_txn() -> Transaction {
    let account1 = Account::new();
    Transaction::UserTransaction(create_account_txn_send_with_association_account(
        &account1, 1, // fix me
        1_000,
    ))
}
