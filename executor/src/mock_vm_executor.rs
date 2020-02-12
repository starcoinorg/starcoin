// Copyright (c) The Libra Core Contributors
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


pub struct MockVM;

impl VMExecutor for MockVM {
    fn execute_block(
        transactions: Vec<Transaction>,
        _config: &VMConfig,
        state_view: &dyn StateView,
    ) -> Result<Vec<TransactionOutput>, VMStatus> {
        unimplemented!()
    }
}