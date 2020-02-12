// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_address::AccountAddress;
use crate::block_metadata::BlockMetadata;
use crate::transaction::SignedUserTransaction;
use libra_crypto::{
    hash::{CryptoHash, CryptoHasher},
    HashValue,
};
use libra_crypto_derive::CryptoHasher;
use serde::{Deserialize, Serialize};

/// Type for block number.
pub type BlockNumber = u64;

#[derive(Default, Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, CryptoHasher)]
pub struct BlockHeader {
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
    /// The last transaction state_root of this block after execute.
    state_root: HashValue,
    /// Gas used for contracts execution.
    gas_used: u64,
    /// Block gas limit.
    gas_limit: u64,
    /// Block proof of work extend field.
    pow: Vec<u8>,
}

impl BlockHeader {
    pub fn id(&self) -> HashValue {
        self.hash()
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

    pub fn pow(&self) -> &[u8] {
        self.pow.as_slice()
    }

    pub fn into_metadata(self) -> BlockMetadata {
        BlockMetadata::new(self.id(), self.timestamp, self.author)
    }
}

impl CryptoHash for BlockHeader {
    type Hasher = BlockHeaderHasher;

    fn hash(&self) -> HashValue {
        let mut state = Self::Hasher::default();
        state.write(
            scs::to_bytes(self)
                .expect("Failed to serialize BlockHeader")
                .as_slice(),
        );
        state.finish()
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
}
