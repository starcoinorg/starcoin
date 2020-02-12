// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::repository::Repository;
use anyhow::Result;
use state_view::StateView;
use types::{
    block::Block,
    transaction::{SignedTransaction, TransactionOutput},
};

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
