// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{Error, Result};
use types::transaction::SignedUserTransaction;

#[async_trait::async_trait]
pub trait TxPool: Clone + std::marker::Unpin {
    async fn add(self, txn: SignedUserTransaction) -> Result<bool>;
    async fn get_pending_txns(self) -> Result<Vec<SignedUserTransaction>>;
}
