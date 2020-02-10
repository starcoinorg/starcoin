// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_address::AccountAddress;
use crate::transaction::SignedTransaction;
use libra_crypto::HashValue;
use serde::{Deserialize, Serialize};

/// Type for block number.
pub type BlockNumber = u64;

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct Header {
    /// Parent hash.
    parent_hash: HashValue,
    /// Block timestamp.
    timestamp: u64,
    /// Block number.
    number: BlockNumber,
    /// Block author.
    author: AccountAddress,
    /// Transactions root.
    transactions_root: HashValue,
    /// The accumulator root hash after executing this block.
    accumulator_root: HashValue,
    /// Gas used for contracts execution.
    gas_used: u64,
    /// Block gas limit.
    gas_limit: u64,
    /// Block proof of work extend field.
    pow: Vec<u8>,
}

/// A block, encoded as it is on the block chain.
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct Block {
    /// The header of this block.
    pub header: Header,
    /// The transactions in this block.
    pub transactions: Vec<SignedTransaction>,
}
