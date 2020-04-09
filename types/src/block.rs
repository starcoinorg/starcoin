// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_address::AccountAddress;
use crate::block_metadata::BlockMetadata;
use crate::transaction::SignedUserTransaction;
use starcoin_crypto::{hash::CryptoHash, HashValue};

use crate::U256;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use starcoin_crypto::hash::create_literal_hash;
use std::cmp::Ordering;
use std::cmp::PartialOrd;

/// Type for block number.
pub type BlockNumber = u64;
/// Type for branch number.
pub type BranchNumber = (HashValue, u64);

#[derive(
    Default, Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Serialize, Deserialize, CryptoHash,
)]
pub struct BlockHeader {
    /// Parent hash.
    parent_hash: HashValue,
    /// Block timestamp.
    timestamp: u64,
    /// Block number.
    number: BlockNumber,
    /// Block author.
    author: AccountAddress,
    /// auth_key_prefix for create_account
    auth_key_prefix: Option<Vec<u8>>,
    /// The accumulator root hash after executing this block.
    accumulator_root: HashValue,
    /// The last transaction state_root of this block after execute.
    state_root: HashValue,
    /// Gas used for contracts execution.
    gas_used: u64,
    /// Block gas limit.
    gas_limit: u64,
    /// Block difficult
    difficult: U256,
    /// Consensus extend header field.
    //TODO: change to ConsensusHeader
    consensus_header: Vec<u8>,
}

impl BlockHeader {
    pub fn new<H>(
        parent_hash: HashValue,
        timestamp: u64,
        number: BlockNumber,
        author: AccountAddress,
        accumulator_root: HashValue,
        state_root: HashValue,
        gas_used: u64,
        gas_limit: u64,
        difficult: U256,
        consensus_header: H,
    ) -> BlockHeader
    where
        H: Into<Vec<u8>>,
    {
        Self::new_with_auth(
            parent_hash,
            timestamp,
            number,
            author,
            None,
            accumulator_root,
            state_root,
            gas_used,
            gas_limit,
            difficult,
            consensus_header,
        )
    }

    pub fn new_with_auth<H>(
        parent_hash: HashValue,
        timestamp: u64,
        number: BlockNumber,
        author: AccountAddress,
        auth_key_prefix: Option<Vec<u8>>,
        accumulator_root: HashValue,
        state_root: HashValue,
        gas_used: u64,
        gas_limit: u64,
        difficult: U256,
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
            auth_key_prefix,
            accumulator_root,
            state_root,
            gas_used,
            gas_limit,
            difficult,
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
        BlockMetadata::new(
            self.parent_hash(),
            self.timestamp,
            self.author,
            self.auth_key_prefix,
        )
    }
    pub fn difficult(&self) -> U256 {
        self.difficult
    }

    pub fn genesis_block_header(
        accumulator_root: HashValue,
        state_root: HashValue,
        difficult: U256,
        consensus_header: Vec<u8>,
    ) -> Self {
        Self {
            //TODO should use a placeholder hash?
            parent_hash: HashValue::zero(),
            //TODO hard code a genesis block time.
            timestamp: 0,
            number: 0,
            author: AccountAddress::default(),
            auth_key_prefix: None,
            accumulator_root,
            state_root,
            gas_used: 0,
            //TODO
            gas_limit: 0,
            difficult,
            consensus_header,
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

#[derive(Default, Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct BlockBody {
    /// The transactions in this block.
    transactions: Vec<SignedUserTransaction>,
}

impl BlockBody {
    pub fn new(transactions: Vec<SignedUserTransaction>) -> Self {
        Self { transactions }
    }
}

impl Into<BlockBody> for Vec<SignedUserTransaction> {
    fn into(self) -> BlockBody {
        BlockBody { transactions: self }
    }
}

impl Into<Vec<SignedUserTransaction>> for BlockBody {
    fn into(self) -> Vec<SignedUserTransaction> {
        self.transactions
    }
}

/// A block, encoded as it is on the block chain.
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, CryptoHash)]
pub struct Block {
    /// The header of this block.
    header: BlockHeader,
    /// The body of this block.
    body: BlockBody,
}

impl Block {
    pub fn new<B>(header: BlockHeader, body: B) -> Self
    where
        B: Into<BlockBody>,
    {
        Block {
            header,
            body: body.into(),
        }
    }

    pub fn header(&self) -> &BlockHeader {
        &self.header
    }
    pub fn transactions(&self) -> &[SignedUserTransaction] {
        self.body.transactions.as_slice()
    }
    pub fn into_inner(self) -> (BlockHeader, BlockBody) {
        (self.header, self.body)
    }

    pub fn genesis_block(
        accumulator_root: HashValue,
        state_root: HashValue,
        difficult: U256,
        consensus_header: Vec<u8>,
    ) -> Self {
        let header = BlockHeader::genesis_block_header(
            accumulator_root,
            state_root,
            difficult,
            consensus_header,
        );
        Self {
            header,
            body: BlockBody::default(),
        }
    }
}

/// Default ID of `BlockInfo`.
pub static BLOCK_INFO_DEFAULT_ID: Lazy<HashValue> =
    Lazy::new(|| create_literal_hash("BLOCK_INFO_DEFAULT_ID"));

/// `BlockInfo` is the object we store in the storage. It consists of the
/// block as well as the execution result of this block.
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, CryptoHash)]
pub struct BlockInfo {
    /// Block id
    pub block_id: HashValue,
    /// Frozen subtree roots of this accumulator.
    pub frozen_subtree_roots: Vec<HashValue>,
    /// The total number of leaves in this accumulator.
    pub num_leaves: u64,
    /// The total number of nodes in this accumulator.
    pub num_nodes: u64,
}

impl BlockInfo {
    pub fn new(
        block_id: HashValue,
        frozen_subtree_roots: Vec<HashValue>,
        num_leaves: u64,
        num_nodes: u64,
    ) -> Self {
        Self {
            block_id,
            frozen_subtree_roots,
            num_leaves,
            num_nodes,
        }
    }
    pub fn into_inner(self) -> (HashValue, Vec<HashValue>, u64, u64) {
        self.into()
    }

    pub fn id(&self) -> HashValue {
        self.crypto_hash()
    }

    pub fn get_total_difficult(&self) -> U256 {
        U256::from(0)
    }
}

impl Into<(HashValue, Vec<HashValue>, u64, u64)> for BlockInfo {
    fn into(self) -> (HashValue, Vec<HashValue>, u64, u64) {
        (
            self.block_id,
            self.frozen_subtree_roots,
            self.num_leaves,
            self.num_nodes,
        )
    }
}

#[derive(Clone)]
pub struct BlockTemplate {
    /// Parent hash.
    pub parent_hash: HashValue,
    /// Block timestamp.
    pub timestamp: u64,
    /// Block number.
    pub number: BlockNumber,
    /// Block author.
    pub author: AccountAddress,
    /// auth_key_prefix
    pub auth_key_prefix: Option<Vec<u8>>,
    /// The accumulator root hash after executing this block.
    pub accumulator_root: HashValue,
    /// The last transaction state_root of this block after execute.
    pub state_root: HashValue,
    /// Gas used for contracts execution.
    pub gas_used: u64,
    /// Block gas limit.
    pub gas_limit: u64,

    /// Block difficult
    pub difficult: U256,

    pub body: BlockBody,
}

impl BlockTemplate {
    pub fn new(
        parent_hash: HashValue,
        timestamp: u64,
        number: BlockNumber,
        author: AccountAddress,
        auth_key_prefix: Option<Vec<u8>>,
        accumulator_root: HashValue,
        state_root: HashValue,
        gas_used: u64,
        gas_limit: u64,
        difficult: U256,
        body: BlockBody,
    ) -> Self {
        Self {
            parent_hash,
            timestamp,
            number,
            author,
            auth_key_prefix,
            accumulator_root,
            state_root,
            gas_used,
            gas_limit,
            difficult,
            body,
        }
    }

    pub fn into_block<H>(self, consensus_header: H) -> Block
    where
        H: Into<Vec<u8>>,
    {
        let header = BlockHeader::new_with_auth(
            self.parent_hash,
            self.timestamp,
            self.number,
            self.author,
            self.auth_key_prefix,
            self.accumulator_root,
            self.state_root,
            self.gas_used,
            self.gas_limit,
            self.difficult,
            consensus_header.into(),
        );
        Block {
            header,
            body: self.body,
        }
    }
    pub fn into_block_header<H>(self, consensus_header: H) -> BlockHeader
    where
        H: Into<Vec<u8>>,
    {
        let header = BlockHeader::new_with_auth(
            self.parent_hash,
            self.timestamp,
            self.number,
            self.author,
            self.auth_key_prefix,
            self.accumulator_root,
            self.state_root,
            self.gas_used,
            self.gas_limit,
            self.difficult,
            consensus_header.into(),
        );
        header
    }

    pub fn from_block(block: Block) -> Self {
        BlockTemplate {
            parent_hash: block.header().parent_hash,
            timestamp: block.header().timestamp,
            number: block.header().number,
            author: block.header().author,
            auth_key_prefix: block.header().auth_key_prefix.clone(),
            accumulator_root: block.header().accumulator_root,
            state_root: block.header().state_root,
            gas_used: block.header().gas_used,
            gas_limit: block.header().gas_limit,

            difficult: block.header().difficult,
            body: block.body,
        }
    }
}

#[derive(Clone, Debug, Hash, Serialize, Deserialize, CryptoHash)]
pub struct BlockDetail {
    block: Block,
    total_difficulty: U256,
}

impl BlockDetail {
    pub fn new(block: Block, total_difficulty: U256) -> Self {
        BlockDetail {
            block,
            total_difficulty,
        }
    }

    pub fn get_total_difficulty(&self) -> U256 {
        self.total_difficulty
    }

    pub fn get_block(&self) -> &Block {
        &self.block
    }

    pub fn header(&self) -> &BlockHeader {
        self.block.header()
    }
}
