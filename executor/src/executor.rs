// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::TransactionExecutor;
use anyhow::{Error, Result};
use compiler::compile::StarcoinCompiler;
use config::VMConfig;
use crypto::HashValue;
use traits::ChainState;
use types::{
    state_set::ChainStateSet,
    transaction::{SignedUserTransaction, Transaction, TransactionOutput},
    vm_error::VMStatus,
};
use vm_runtime::starcoin_vm::StarcoinVM;

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
    fn init_genesis(config: &VMConfig) -> Result<(HashValue, ChainStateSet)> {
        unimplemented!()
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
        None
    }
}

pub fn mock_txn() -> Transaction {
    let empty_script = StarcoinCompiler::compile_script("main() {return;}");
    Transaction::UserTransaction(SignedUserTransaction::mock_from(empty_script))
}
