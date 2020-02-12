// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_config::config::VMConfig;
use starcoin_state_view::StateView;
use starcoin_types::{
    transaction::{
        Transaction, TransactionOutput,
    },
    vm_error::VMStatus,
};
use vm_runtime::VMExecutor;

pub struct Executor {}

impl Executor {
    pub fn execute_transaction(
        &self,
        repo: &dyn Repository,
        tx: &SignedTransaction,
    ) -> Result<TransactionOutput> {
        unimplemented!()
    }
}
