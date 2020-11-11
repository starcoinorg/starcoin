// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_address::AccountAddress;
use crate::block_metadata::BlockMetadata;
use crate::transaction::SignedUserTransaction;
use starcoin_crypto::{
    hash::{CryptoHash, CryptoHasher, PlainCryptoHash},
    HashValue,
};

use crate::genesis_config::ChainId;
use crate::language_storage::CORE_CODE_ADDRESS;
use crate::U256;
use serde::export::Formatter;
use serde::{Deserialize, Serialize};
use starcoin_accumulator::accumulator_info::AccumulatorInfo;
use starcoin_crypto::hash::ACCUMULATOR_PLACEHOLDER_HASH;
use starcoin_vm_types::transaction::authenticator::AuthenticationKey;

/// Type for block number.
pub type BlockNumber = u64;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
pub struct BlockIdAndNumber {
    pub id: HashValue,
    pub number: BlockNumber,
}

impl BlockIdAndNumber {
    pub fn new(id: HashValue, number: BlockNumber) -> Self {
        Self { id, number }
    }
}

/// block timestamp allowed future times
pub const ALLOWED_FUTURE_BLOCKTIME: u64 = 30000; // 30 second;

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, CryptoHasher, CryptoHash)]
pub struct BlockHeader {
    /// Parent hash.
    pub parent_hash: HashValue,
    /// Block timestamp.
    pub timestamp: u64,
    /// Block number.
    pub number: BlockNumber,
    /// Block author.
    pub author: AccountAddress,
    /// Block author auth key.
    pub author_auth_key: Option<AuthenticationKey>,
    /// The transaction accumulator root hash after executing this block.
    pub accumulator_root: HashValue,
    /// The parent block accumulator root hash.
    pub parent_block_accumulator_root: HashValue,
    /// The last transaction state_root of this block after execute.
    pub state_root: HashValue,
    /// Gas used for contracts execution.
    pub gas_used: u64,
    /// Block difficulty
    pub difficulty: U256,
    /// Consensus nonce field.
    pub nonce: u32,
    /// hash for block body
    pub body_hash: HashValue,
    /// The chain id
    pub chain_id: ChainId,
}

impl BlockHeader {
    pub fn new(
        parent_hash: HashValue,
        parent_block_accumulator_root: HashValue,
        timestamp: u64,
        number: BlockNumber,
        author: AccountAddress,
        accumulator_root: HashValue,
        state_root: HashValue,
        gas_used: u64,
        difficulty: U256,
        nonce: u32,
        body_hash: HashValue,
        chain_id: ChainId,
    ) -> BlockHeader {
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
            difficulty,
            nonce,
            body_hash,
            chain_id,
        )
    }

    pub fn new_with_auth(
        parent_hash: HashValue,
        parent_block_accumulator_root: HashValue,
        timestamp: u64,
        number: BlockNumber,
        author: AccountAddress,
        author_auth_key: Option<AuthenticationKey>,
        accumulator_root: HashValue,
        state_root: HashValue,
        gas_used: u64,
        difficulty: U256,
        nonce: u32,
        body_hash: HashValue,
        chain_id: ChainId,
    ) -> BlockHeader {
        BlockHeader {
            parent_hash,
            parent_block_accumulator_root,
            number,
            timestamp,
            author,
            author_auth_key,
            accumulator_root,
            state_root,
            gas_used,
            difficulty,
            nonce,
            body_hash,
            chain_id,
        }
    }

    pub fn as_pow_header_blob(&self) -> Vec<u8> {
        let mut blob = Vec::new();
        let raw_header: RawBlockHeader = self.to_owned().into();
        let raw_header_hash = raw_header.crypto_hash();
        let mut diff_bytes = [0u8; 32];
        raw_header.difficulty.to_big_endian(&mut diff_bytes);
        let extend_and_nonce = [0u8; 12];
        blob.extend_from_slice(raw_header_hash.to_vec().as_slice());
        blob.extend_from_slice(&extend_and_nonce);
        blob.extend_from_slice(&diff_bytes);
        blob
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

    pub fn nonce(&self) -> u32 {
        self.nonce
    }

    pub fn difficulty(&self) -> U256 {
        self.difficulty
    }

    pub fn parent_block_accumulator_root(&self) -> HashValue {
        self.parent_block_accumulator_root
    }

    pub fn chain_id(&self) -> ChainId {
        self.chain_id
    }
    pub fn is_genesis(&self) -> bool {
        self.number == 0
    }

    pub fn body_hash(&self) -> HashValue {
        self.body_hash
    }
    pub fn genesis_block_header(
        parent_hash: HashValue,
        timestamp: u64,
        accumulator_root: HashValue,
        state_root: HashValue,
        difficulty: U256,
        nonce: u32,
        body_hash: HashValue,
        chain_id: ChainId,
    ) -> Self {
        Self {
            parent_hash,
            parent_block_accumulator_root: *ACCUMULATOR_PLACEHOLDER_HASH,
            timestamp,
            number: 0,
            author: CORE_CODE_ADDRESS,
            author_auth_key: None,
            accumulator_root,
            state_root,
            gas_used: 0,
            difficulty,
            nonce,
            body_hash,
            chain_id,
        }
    }

    pub fn random() -> Self {
        Self {
            parent_hash: HashValue::random(),
            parent_block_accumulator_root: HashValue::random(),
            timestamp: rand::random(),
            number: rand::random(),
            author: AccountAddress::random(),
            author_auth_key: None,
            accumulator_root: HashValue::random(),
            state_root: HashValue::random(),
            gas_used: rand::random(),
            difficulty: U256::max_value(),
            nonce: 0,
            body_hash: HashValue::random(),
            chain_id: ChainId::test(),
        }
    }
}

impl Into<RawBlockHeader> for BlockHeader {
    fn into(self) -> RawBlockHeader {
        RawBlockHeader {
            parent_hash: self.parent_hash,
            timestamp: self.timestamp,
            number: self.number,
            author: self.author,
            author_auth_key: self.author_auth_key,
            accumulator_root: self.accumulator_root,
            parent_block_accumulator_root: self.parent_block_accumulator_root,
            state_root: self.state_root,
            gas_used: self.gas_used,
            difficulty: self.difficulty,
            body_hash: self.body_hash,
            chain_id: self.chain_id,
        }
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, CryptoHasher, CryptoHash)]
pub struct RawBlockHeader {
    /// Parent hash.
    pub parent_hash: HashValue,
    /// Block timestamp.
    pub timestamp: u64,
    /// Block number.
    pub number: BlockNumber,
    /// Block author.
    pub author: AccountAddress,
    /// Block author auth key.
    pub author_auth_key: Option<AuthenticationKey>,
    /// The transaction accumulator root hash after executing this block.
    pub accumulator_root: HashValue,
    /// The parent block accumulator root hash.
    pub parent_block_accumulator_root: HashValue,
    /// The last transaction state_root of this block after execute.
    pub state_root: HashValue,
    /// Gas used for contracts execution.
    pub gas_used: u64,
    /// Block difficulty
    pub difficulty: U256,
    /// hash for block body
    pub body_hash: HashValue,
    /// The chain id
    pub chain_id: ChainId,
}

#[derive(
    Default, Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, CryptoHasher, CryptoHash,
)]
pub struct BlockBody {
    /// The transactions in this block.
    pub transactions: Vec<SignedUserTransaction>,
    /// uncles block header
    pub uncles: Option<Vec<BlockHeader>>,
}

impl BlockBody {
    pub fn new(transactions: Vec<SignedUserTransaction>, uncles: Option<Vec<BlockHeader>>) -> Self {
        Self {
            transactions,
            uncles,
        }
    }
    pub fn get_txn(&self, index: usize) -> Option<&SignedUserTransaction> {
        self.transactions.get(index)
    }

    /// Just for test
    pub fn new_empty() -> BlockBody {
        BlockBody {
            transactions: Vec::new(),
            uncles: None,
        }
    }

    pub fn hash(&self) -> HashValue {
        self.crypto_hash()
    }
}

impl Into<BlockBody> for Vec<SignedUserTransaction> {
    fn into(self) -> BlockBody {
        BlockBody {
            transactions: self,
            uncles: None,
        }
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
    pub fn uncles(&self) -> Option<&[BlockHeader]> {
        match &self.body.uncles {
            Some(uncles) => Some(uncles.as_slice()),
            None => None,
        }
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
        nonce: u32,
        genesis_txn: SignedUserTransaction,
    ) -> Self {
        let chain_id = genesis_txn.chain_id();
        let block_body = BlockBody::new(vec![genesis_txn], None);
        let header = BlockHeader::genesis_block_header(
            parent_hash,
            timestamp,
            accumulator_root,
            state_root,
            difficulty,
            nonce,
            block_body.hash(),
            chain_id,
        );
        Self {
            header,
            body: block_body,
        }
    }

    pub fn to_metadata(&self, parent_gas_used: u64) -> BlockMetadata {
        let uncles = self
            .body
            .uncles
            .as_ref()
            .map(|uncles| uncles.len() as u64)
            .unwrap_or(0);

        BlockMetadata::new(
            self.header.parent_hash(),
            self.header.timestamp,
            self.header.author,
            self.header.author_auth_key,
            uncles,
            self.header.number,
            self.header.chain_id,
            parent_gas_used,
        )
    }
}

impl std::fmt::Display for Block {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Block{{id:\"{}\", number:\"{}\", parent_id:\"{}\",",
            self.id(),
            self.header().number(),
            self.header().parent_hash()
        )?;
        if let Some(uncles) = &self.body.uncles {
            write!(f, "uncles:[")?;
            for uncle in uncles {
                write!(f, "\"{}\",", uncle.id())?;
            }
            write!(f, "],")?;
        }
        write!(f, "transactions:[")?;
        for txn in &self.body.transactions {
            write!(f, "\"{}\",", txn.id())?;
        }
        write!(f, "]}}")
    }
}

/// `BlockInfo` is the object we store in the storage. It consists of the
/// block as well as the execution result of this block.
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, CryptoHasher, CryptoHash)]
pub struct BlockInfo {
    /// Block id
    pub block_id: HashValue,
    /// The transaction accumulator info
    pub txn_accumulator_info: AccumulatorInfo,
    /// The total difficulty.
    pub total_difficulty: U256,
    /// The block accumulator info.
    pub block_accumulator_info: AccumulatorInfo,
}

impl BlockInfo {
    pub fn new(
        block_id: HashValue,
        txn_accumulator_info: AccumulatorInfo,
        total_difficulty: U256,
        block_accumulator_info: AccumulatorInfo,
    ) -> Self {
        Self {
            block_id,
            txn_accumulator_info,
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
            txn_accumulator_info,
            total_difficulty,
            block_accumulator_info,
        }
    }

    pub fn into_inner(self) -> (HashValue, AccumulatorInfo, U256, AccumulatorInfo) {
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

    pub fn get_txn_accumulator_info(&self) -> &AccumulatorInfo {
        &self.txn_accumulator_info
    }

    pub fn block_id(&self) -> &HashValue {
        &self.block_id
    }
}

impl Into<(HashValue, AccumulatorInfo, U256, AccumulatorInfo)> for BlockInfo {
    fn into(self) -> (HashValue, AccumulatorInfo, U256, AccumulatorInfo) {
        (
            self.block_id,
            self.txn_accumulator_info,
            self.total_difficulty,
            self.block_accumulator_info,
        )
    }
}

#[derive(Clone, Debug)]
pub struct BlockTemplate {
    /// Parent hash.
    pub parent_hash: HashValue,
    /// Block timestamp.
    pub timestamp: u64,
    /// Block number.
    pub number: BlockNumber,
    /// Block author.
    pub author: AccountAddress,
    /// Block author auth key.
    pub author_auth_key: Option<AuthenticationKey>,
    /// The accumulator root hash after executing this block.
    pub accumulator_root: HashValue,
    /// The parent block accumulator root hash.
    pub parent_block_accumulator_root: HashValue,
    /// The last transaction state_root of this block after execute.
    pub state_root: HashValue,
    /// Gas used for contracts execution.
    pub gas_used: u64,
    /// hash for block body
    pub body_hash: HashValue,
    pub body: BlockBody,
    /// The chain id
    pub chain_id: ChainId,
}

impl BlockTemplate {
    pub fn new(
        parent_hash: HashValue,
        parent_block_accumulator_root: HashValue,
        timestamp: u64,
        number: BlockNumber,
        author: AccountAddress,
        author_auth_key: Option<AuthenticationKey>,
        accumulator_root: HashValue,
        state_root: HashValue,
        gas_used: u64,
        body_hash: HashValue,
        body: BlockBody,
        chain_id: ChainId,
    ) -> Self {
        Self {
            parent_hash,
            parent_block_accumulator_root,
            timestamp,
            number,
            author,
            author_auth_key,
            accumulator_root,
            state_root,
            gas_used,
            body_hash,
            body,
            chain_id,
        }
    }

    pub fn into_block(self, nonce: u32, difficulty: U256) -> Block {
        let header = BlockHeader::new_with_auth(
            self.parent_hash,
            self.parent_block_accumulator_root,
            self.timestamp,
            self.number,
            self.author,
            self.author_auth_key,
            self.accumulator_root,
            self.state_root,
            self.gas_used,
            difficulty,
            nonce,
            self.body_hash,
            self.chain_id,
        );
        Block {
            header,
            body: self.body,
        }
    }

    pub fn as_raw_block_header(&self, difficulty: U256) -> RawBlockHeader {
        RawBlockHeader {
            parent_hash: self.parent_hash,
            timestamp: self.timestamp,
            number: self.number,
            author: self.author,
            author_auth_key: self.author_auth_key,
            accumulator_root: self.accumulator_root,
            parent_block_accumulator_root: self.parent_block_accumulator_root,
            state_root: self.state_root,
            gas_used: self.gas_used,
            body_hash: self.body_hash,
            difficulty,
            chain_id: self.chain_id,
        }
    }

    pub fn as_pow_header_blob(&self, difficulty: U256) -> Vec<u8> {
        let mut blob = Vec::new();
        let raw_header = self.as_raw_block_header(difficulty);
        let raw_header_hash = raw_header.crypto_hash();
        let mut dh = [0u8; 32];
        difficulty.to_big_endian(&mut dh);
        let extend_and_nonce = [0u8; 12];

        blob.extend_from_slice(raw_header_hash.to_vec().as_slice());
        blob.extend_from_slice(&extend_and_nonce);
        blob.extend_from_slice(&dh);
        blob
    }

    pub fn into_block_header(self, nonce: u32, difficulty: U256) -> BlockHeader {
        BlockHeader::new_with_auth(
            self.parent_hash,
            self.parent_block_accumulator_root,
            self.timestamp,
            self.number,
            self.author,
            self.author_auth_key,
            self.accumulator_root,
            self.state_root,
            self.gas_used,
            difficulty,
            nonce,
            self.body_hash,
            self.chain_id,
        )
    }

    pub fn from_block(block: Block) -> Self {
        BlockTemplate {
            parent_hash: block.header().parent_hash,
            parent_block_accumulator_root: block.header().parent_block_accumulator_root(),
            timestamp: block.header().timestamp,
            number: block.header().number,
            author: block.header().author,
            author_auth_key: block.header().author_auth_key,
            accumulator_root: block.header().accumulator_root,
            state_root: block.header().state_root,
            gas_used: block.header().gas_used,
            body: block.body,
            body_hash: block.header.body_hash,
            chain_id: block.header.chain_id,
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
