// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_address::AccountAddress;
use crate::block_metadata::BlockMetadata;
use crate::transaction::SignedUserTransaction;
use starcoin_crypto::{
    hash::{CryptoHash, CryptoHasher, PlainCryptoHash},
    HashValue,
};

use crate::accumulator_info::AccumulatorInfo;
use crate::language_storage::CORE_CODE_ADDRESS;
use crate::U256;
use serde::{Deserialize, Serialize};
use starcoin_accumulator::node::ACCUMULATOR_PLACEHOLDER_HASH;
use std::cmp::Ordering;
use std::cmp::PartialOrd;

/// Type for block number.
pub type BlockNumber = u64;
/// block timestamp allowed future times
pub const ALLOWED_FUTURE_BLOCKTIME: u64 = 15 * 1000; // 15 Second;

#[derive(
    Default,
    Clone,
    Debug,
    Hash,
    Eq,
    PartialEq,
    PartialOrd,
    Serialize,
    Deserialize,
    CryptoHasher,
    CryptoHash,
)]
pub struct BlockHeader {
    /// Parent hash.
    pub parent_hash: HashValue,
    /// Block timestamp.
    pub timestamp: u64,
    /// Block number.
    pub number: BlockNumber,
    /// Block author.
    pub author: AccountAddress,
    /// auth_key_prefix for create_account
    pub auth_key_prefix: Option<Vec<u8>>,
    /// The transaction accumulator root hash after executing this block.
    pub accumulator_root: HashValue,
    /// The parent block accumulator root hash.
    pub parent_block_accumulator_root: HashValue,
    /// The last transaction state_root of this block after execute.
    pub state_root: HashValue,
    /// Gas used for contracts execution.
    pub gas_used: u64,
    /// Block gas limit.
    pub gas_limit: u64,
    /// Block difficulty
    pub difficulty: U256,
    /// Consensus extend header field.
    pub consensus_header: Vec<u8>,
}

impl BlockHeader {
    pub fn new<H>(
        parent_hash: HashValue,
        parent_block_accumulator_root: HashValue,
        timestamp: u64,
        number: BlockNumber,
        author: AccountAddress,
        accumulator_root: HashValue,
        state_root: HashValue,
        gas_used: u64,
        gas_limit: u64,
        difficulty: U256,
        consensus_header: H,
    ) -> BlockHeader
    where
        H: Into<Vec<u8>>,
    {
        Self::new_with_auth(
            parent_hash,
            parent_block_accumulator_root,
            timestamp,
            number,
            author,
            None,
            accumulator_root,
            state_root,
            gas_used,
            gas_limit,
            difficulty,
            consensus_header,
        )
    }

    pub fn new_with_auth<H>(
        parent_hash: HashValue,
        parent_block_accumulator_root: HashValue,
        timestamp: u64,
        number: BlockNumber,
        author: AccountAddress,
        auth_key_prefix: Option<Vec<u8>>,
        accumulator_root: HashValue,
        state_root: HashValue,
        gas_used: u64,
        gas_limit: u64,
        difficulty: U256,
        consensus_header: H,
    ) -> BlockHeader
    where
        H: Into<Vec<u8>>,
    {
        BlockHeader {
            parent_hash,
            parent_block_accumulator_root,
            number,
            timestamp,
            author,
            auth_key_prefix,
            accumulator_root,
            state_root,
            gas_used,
            gas_limit,
            difficulty,
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
    pub fn difficulty(&self) -> U256 {
        self.difficulty
    }

    pub fn parent_block_accumulator_root(&self) -> HashValue {
        self.parent_block_accumulator_root
    }

    pub fn genesis_block_header(
        parent_hash: HashValue,
        timestamp: u64,
        accumulator_root: HashValue,
        state_root: HashValue,
        difficulty: U256,
        consensus_header: Vec<u8>,
    ) -> Self {
        Self {
            parent_hash,
            parent_block_accumulator_root: *ACCUMULATOR_PLACEHOLDER_HASH,
            timestamp,
            number: 0,
            author: CORE_CODE_ADDRESS,
            auth_key_prefix: None,
            accumulator_root,
            state_root,
            gas_used: 0,
            gas_limit: 0,
            difficulty,
            consensus_header,
        }
    }

    pub fn random() -> Self {
        Self {
            parent_hash: HashValue::random(),
            parent_block_accumulator_root: HashValue::random(),
            timestamp: rand::random(),
            number: rand::random(),
            author: AccountAddress::random(),
            auth_key_prefix: None,
            accumulator_root: HashValue::random(),
            state_root: HashValue::random(),
            gas_used: rand::random(),
            gas_limit: rand::random(),
            difficulty: U256::max_value(),
            consensus_header: vec![],
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
            Ordering::Equal => self.gas_used.cmp(&other.gas_used).reverse(),
            ordering => ordering,
        }
    }
}

impl Into<BlockMetadata> for BlockHeader {
    fn into(self) -> BlockMetadata {
        self.into_metadata()
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
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, CryptoHasher, CryptoHash)]
pub struct Block {
    /// The header of this block.
    pub header: BlockHeader,
    /// The body of this block.
    pub body: BlockBody,
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

    pub fn id(&self) -> HashValue {
        self.header.id()
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
        parent_hash: HashValue,
        timestamp: u64,
        accumulator_root: HashValue,
        state_root: HashValue,
        difficulty: U256,
        consensus_header: Vec<u8>,
        genesis_txn: SignedUserTransaction,
    ) -> Self {
        let header = BlockHeader::genesis_block_header(
            parent_hash,
            timestamp,
            accumulator_root,
            state_root,
            difficulty,
            consensus_header,
        );
        Self {
            header,
            body: BlockBody::new(vec![genesis_txn]),
        }
    }
}

/// `BlockInfo` is the object we store in the storage. It consists of the
/// block as well as the execution result of this block.
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, CryptoHasher, CryptoHash)]
pub struct BlockInfo {
    /// Block id
    pub block_id: HashValue,
    //TODO group txn accumulator's fields.
    /// Accumulator root hash
    pub accumulator_root: HashValue,
    /// Frozen subtree roots of this accumulator.
    pub frozen_subtree_roots: Vec<HashValue>,
    /// The total number of leaves in this accumulator.
    pub num_leaves: u64,
    /// The total number of nodes in this accumulator.
    pub num_nodes: u64,
    /// The total difficulty.
    pub total_difficulty: U256,
    /// The block accumulator info.
    pub block_accumulator_info: AccumulatorInfo,
}

impl BlockInfo {
    pub fn new(
        block_id: HashValue,
        accumulator_root: HashValue,
        frozen_subtree_roots: Vec<HashValue>,
        num_leaves: u64,
        num_nodes: u64,
        total_difficulty: U256,
        block_accumulator_info: AccumulatorInfo,
    ) -> Self {
        Self {
            block_id,
            accumulator_root,
            frozen_subtree_roots,
            num_leaves,
            num_nodes,
            total_difficulty,
            block_accumulator_info,
        }
    }

    pub fn new_with_accumulator_info(
        block_id: HashValue,
        txn_accumulator_info: AccumulatorInfo,
        block_accumulator_info: AccumulatorInfo,
        total_difficulty: U256,
    ) -> Self {
        Self {
            block_id,
            accumulator_root: *txn_accumulator_info.get_accumulator_root(),
            frozen_subtree_roots: txn_accumulator_info.get_frozen_subtree_roots().clone(),
            num_leaves: txn_accumulator_info.get_num_leaves(),
            num_nodes: txn_accumulator_info.get_num_nodes(),
            total_difficulty,
            block_accumulator_info,
        }
    }

    pub fn into_inner(
        self,
    ) -> (
        HashValue,
        HashValue,
        Vec<HashValue>,
        u64,
        u64,
        U256,
        AccumulatorInfo,
    ) {
        self.into()
    }

    pub fn id(&self) -> HashValue {
        self.crypto_hash()
    }

    pub fn get_total_difficulty(&self) -> U256 {
        self.total_difficulty
    }

    pub fn get_block_accumulator_info(&self) -> &AccumulatorInfo {
        &self.block_accumulator_info
    }

    pub fn get_txn_accumulator_info(&self) -> AccumulatorInfo {
        AccumulatorInfo::new(
            self.accumulator_root,
            self.frozen_subtree_roots.clone(),
            self.num_leaves,
            self.num_nodes,
        )
    }

    pub fn block_id(&self) -> &HashValue {
        &self.block_id
    }
}

impl
    Into<(
        HashValue,
        HashValue,
        Vec<HashValue>,
        u64,
        u64,
        U256,
        AccumulatorInfo,
    )> for BlockInfo
{
    fn into(
        self,
    ) -> (
        HashValue,
        HashValue,
        Vec<HashValue>,
        u64,
        u64,
        U256,
        AccumulatorInfo,
    ) {
        (
            self.block_id,
            self.accumulator_root,
            self.frozen_subtree_roots,
            self.num_leaves,
            self.num_nodes,
            self.total_difficulty,
            self.block_accumulator_info,
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
    /// The parent block accumulator root hash.
    pub parent_block_accumulator_root: HashValue,
    /// The last transaction state_root of this block after execute.
    pub state_root: HashValue,
    /// Gas used for contracts execution.
    pub gas_used: u64,
    /// Block gas limit.
    pub gas_limit: u64,

    pub body: BlockBody,
}

impl BlockTemplate {
    pub fn new(
        parent_hash: HashValue,
        parent_block_accumulator_root: HashValue,
        timestamp: u64,
        number: BlockNumber,
        author: AccountAddress,
        auth_key_prefix: Option<Vec<u8>>,
        accumulator_root: HashValue,
        state_root: HashValue,
        gas_used: u64,
        gas_limit: u64,
        body: BlockBody,
    ) -> Self {
        Self {
            parent_hash,
            parent_block_accumulator_root,
            timestamp,
            number,
            author,
            auth_key_prefix,
            accumulator_root,
            state_root,
            gas_used,
            gas_limit,
            body,
        }
    }

    pub fn into_block<H>(self, consensus_header: H, difficulty: U256) -> Block
    where
        H: Into<Vec<u8>>,
    {
        let header = BlockHeader::new_with_auth(
            self.parent_hash,
            self.parent_block_accumulator_root,
            self.timestamp,
            self.number,
            self.author,
            self.auth_key_prefix,
            self.accumulator_root,
            self.state_root,
            self.gas_used,
            self.gas_limit,
            difficulty,
            consensus_header.into(),
        );
        Block {
            header,
            body: self.body,
        }
    }
    pub fn into_block_header<H>(self, consensus_header: H, difficulty: U256) -> BlockHeader
    where
        H: Into<Vec<u8>>,
    {
        BlockHeader::new_with_auth(
            self.parent_hash,
            self.parent_block_accumulator_root,
            self.timestamp,
            self.number,
            self.author,
            self.auth_key_prefix,
            self.accumulator_root,
            self.state_root,
            self.gas_used,
            self.gas_limit,
            difficulty,
            consensus_header.into(),
        )
    }

    pub fn from_block(block: Block) -> Self {
        BlockTemplate {
            parent_hash: block.header().parent_hash,
            parent_block_accumulator_root: block.header().parent_block_accumulator_root(),
            timestamp: block.header().timestamp,
            number: block.header().number,
            author: block.header().author,
            auth_key_prefix: block.header().auth_key_prefix.clone(),
            accumulator_root: block.header().accumulator_root,
            state_root: block.header().state_root,
            gas_used: block.header().gas_used,
            gas_limit: block.header().gas_limit,
            body: block.body,
        }
    }
}

#[derive(Clone, Debug, Hash, Serialize, Deserialize, CryptoHasher, CryptoHash)]
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

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum BlockState {
    Executed,
    Verified,
}
