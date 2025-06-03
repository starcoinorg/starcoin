// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::transaction::TransactionInfo;
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;
use std::ops::Deref;

/// `RichTransactionInfo` is a wrapper of `TransactionInfo` with more info,
/// such as `block_id`, `block_number` which is the block that include the txn producing the txn info.
/// We cannot put the block_id into txn_info, because txn_info is accumulated into block header.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RichTransactionInfo {
    pub block_id: HashValue,
    pub block_number: u64,
    pub transaction_info: TransactionInfo,
    /// Transaction index in block
    pub transaction_index: u32,
    /// Transaction global index in chain, equivalent to transaction accumulator's leaf index
    pub transaction_global_index: u64,
}

impl Deref for RichTransactionInfo {
    type Target = TransactionInfo;

    fn deref(&self) -> &Self::Target {
        &self.transaction_info
    }
}

impl RichTransactionInfo {
    pub fn new(
        block_id: HashValue,
        block_number: u64,
        transaction_info: TransactionInfo,
        transaction_index: u32,
        transaction_global_index: u64,
    ) -> Self {
        Self {
            block_id,
            block_number,
            transaction_info,
            transaction_index,
            transaction_global_index,
        }
    }

    pub fn txn_info(&self) -> &TransactionInfo {
        &self.transaction_info
    }
}
