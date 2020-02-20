// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod chain;
pub mod mock;
mod txpool;

pub use chain::Chain;
use crypto::HashValue;
pub use txpool::TxPool;
use types::ids::TransactionId;
/// Provides various information on a transaction by it's ID
pub trait TransactionInfo {
    /// Get the hash of block that contains the transaction, if any.
    fn transaction_block(&self, id: TransactionId) -> Option<HashValue>;
}
