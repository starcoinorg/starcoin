// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_address::AccountAddress;
use crate::block_metadata::BlockMetadata;
use crate::transaction::SignedUserTransaction;
use crypto::{hash::CryptoHash, HashValue};

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::cmp::PartialOrd;

/// Type for block number.
pub type BlockNumber = u64;

#[derive(Default, Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct BlockHeader {
    /// Parent hash.
    parent_hash: HashValue,
    /// Block timestamp.
    timestamp: u64,
    /// Block number.
    number: BlockNumber,
    /// Block author.
    author: AccountAddress,
    /// The accumulator root hash after executing this block.
    accumulator_root: HashValue,
    /// The last transaction state_root of this block after execute.
    state_root: HashValue,
    /// Gas used for contracts execution.
    gas_used: u64,
    /// Block gas limit.
    gas_limit: u64,
    /// Consensus extend header field.
    consensus_header: Vec<u8>,
}

impl BlockHeader {
    pub fn new<H>(
        parent_hash: HashValue,
        number: BlockNumber,
        timestamp: u64,
        author: AccountAddress,
        accumulator_root: HashValue,
        state_root: HashValue,
        gas_used: u64,
        gas_limit: u64,
        consensus_header: H,
    ) -> BlockHeader
    where
        H: Into<Vec<u8>>,
    {
        BlockHeader {
            parent_hash,
            number,
            timestamp,
            author,
            accumulator_root: HashValue::zero(),
            state_root: HashValue::zero(),
            gas_used: 0,
            gas_limit: 0,
            consensus_header: consensus_header.into(),
        }
    }

    pub fn id(&self) -> HashValue {
        self.crypto_hash()
    }

    pub fn parent_hash(&self) -> HashValue {
        self.parent_hash
    }

    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    pub fn number(&self) -> BlockNumber {
        self.number
    }

    pub fn author(&self) -> AccountAddress {
        self.author
    }

    pub fn accumulator_root(&self) -> HashValue {
        self.accumulator_root
    }

    pub fn state_root(&self) -> HashValue {
        self.state_root
    }

    pub fn gas_used(&self) -> u64 {
        self.gas_used
    }

    pub fn gas_limit(&self) -> u64 {
        self.gas_limit
    }

    pub fn consensus_header(&self) -> &[u8] {
        self.consensus_header.as_slice()
    }

    pub fn into_metadata(self) -> BlockMetadata {
        BlockMetadata::new(self.id(), self.timestamp, self.author)
    }

    #[cfg(any(test))]
    pub fn new_block_for_test(parent_hash: HashValue, parent_height: BlockNumber) -> Self {
        BlockHeader {
            parent_hash,
            timestamp: parent_height + 1,
            /// Block number.
            number: parent_height + 1,
            /// Block author.
            author: AccountAddress::random(),
            /// Transactions root.
            transactions_root: HashValue::random(),
            /// The accumulator root hash after executing this block.
            accumulator_root: HashValue::random(),
            /// The last transaction state_root of this block after execute.
            state_root: HashValue::random(),
            /// Gas used for contracts execution.
            gas_used: 0,
            /// Block gas limit.
            gas_limit: std::u64::MAX(),
            /// Block proof of work extend field.
            pow: HashValue::random().to_vec(),
        }
    }
}

impl Ord for BlockHeader {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.number.cmp(&other.number) {
            Ordering::Equal => {}
            ordering => return ordering,
        }
        match self.timestamp.cmp(&other.timestamp) {
            Ordering::Equal => return self.gas_used.cmp(&other.gas_used).reverse(),
            ordering => return ordering,
        }
    }
}

/// A block, encoded as it is on the block chain.
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct Block {
    /// The header of this block.
    header: BlockHeader,
    /// The transactions in this block.
    transactions: Vec<SignedUserTransaction>,
}

impl Block {
    pub fn header(&self) -> &BlockHeader {
        &self.header
    }
    pub fn transactions(&self) -> &[SignedUserTransaction] {
        self.transactions.as_slice()
    }
    pub fn into_inner(self) -> (BlockHeader, Vec<SignedUserTransaction>) {
        (self.header, self.transactions)
    }

    #[cfg(any(test))]
    pub fn new_nil_block_for_test(header: BlockHeader) -> Self {
        Block {
            header,
            transactions: Vec::new(),
        }
    }
}
