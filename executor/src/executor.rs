// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::TransactionExecutor;
use anyhow::{Result};

use config::VMConfig;
use crypto::HashValue;
use types::{
    state_set::ChainStateSet,
    transaction::{SignedUserTransaction, Transaction, TransactionOutput, TransactionPayload},
    vm_error::VMStatus,
};
use vm_runtime::starcoin_vm::StarcoinVM;
use vm_runtime::genesis::{ generate_genesis_state_set, GENESIS_KEYPAIR, };
use statedb::ChainStateDB;
use std::sync::Arc;
use traits::{ChainState, ChainStateReader, ChainStateWriter};


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
        let repo = Arc::new(storage::memory_storage::MemoryStorage::new());
        let mut chain_state =
            ChainStateDB::new(Arc::new(storage::StarcoinStorage::new(repo).unwrap()), None);

        // ToDo: load genesis txn from genesis.blob, instead of generating from stdlib
        generate_genesis_state_set( &GENESIS_KEYPAIR.0,
                                    GENESIS_KEYPAIR.1.clone(), &chain_state)
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
