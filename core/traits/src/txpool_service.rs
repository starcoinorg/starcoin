// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{Error, Result};
use types::transaction;
use types::transaction::SignedUserTransaction;
#[async_trait::async_trait]
pub trait TxPoolAsyncService: Clone + std::marker::Unpin {
    /// TODO: should be deprecated, use add_txns instead.
    async fn add(self, txn: SignedUserTransaction) -> Result<bool>;
    /// Add all the `txns` into txn pool
    async fn add_txns(
        self,
        txns: Vec<SignedUserTransaction>,
    ) -> Result<Vec<Result<(), transaction::TransactionError>>>;
    /// Get all pending txns which is ok to be packaged to mining.
    async fn get_pending_txns(self, max_len: Option<u64>) -> Result<Vec<SignedUserTransaction>>;
}
