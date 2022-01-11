// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_address::AccountAddress;
use crate::block_metadata::BlockMetadata;
use crate::genesis_config::{ChainId, ConsensusStrategy};
use crate::language_storage::CORE_CODE_ADDRESS;
use crate::transaction::SignedUserTransaction;
use crate::U256;
use bcs_ext::Sample;
use schemars::{self, JsonSchema};
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
pub use starcoin_accumulator::accumulator_info::AccumulatorInfo;
use starcoin_crypto::hash::{ACCUMULATOR_PLACEHOLDER_HASH, SPARSE_MERKLE_PLACEHOLDER_HASH};
use starcoin_crypto::{
    hash::{CryptoHash, CryptoHasher, PlainCryptoHash},
    HashValue,
};
use starcoin_vm_types::account_config::genesis_address;
use starcoin_vm_types::transaction::authenticator::AuthenticationKey;
use std::fmt::Formatter;
/// Type for block number.
pub type BlockNumber = u64;

/// Type for block header extra
#[derive(Clone, Default, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, JsonSchema)]
pub struct BlockHeaderExtra(#[schemars(with = "String")] [u8; 4]);

impl BlockHeaderExtra {
    pub fn new(extra: [u8; 4]) -> Self {
        Self(extra)
    }
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
    pub fn as_slice(&self) -> &[u8; 4] {
        &self.0
    }
}

impl std::fmt::Display for BlockHeaderExtra {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl<'de> Deserialize<'de> for BlockHeaderExtra {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let s = <String>::deserialize(deserializer)?;
            let literal = s.strip_prefix("0x").unwrap_or(&s);
            if literal.len() != 8 {
                return Err(D::Error::custom("Invalid block header extra len"));
            }
            let result = hex::decode(literal).map_err(D::Error::custom)?;
            if result.len() != 4 {
                return Err(D::Error::custom("Invalid block header extra len"));
            }
            let mut extra = [0u8; 4];
            extra.copy_from_slice(&result);
            Ok(BlockHeaderExtra::new(extra))
        } else {
            #[derive(::serde::Deserialize)]
            #[serde(rename = "BlockHeaderExtra")]
            struct Value([u8; 4]);
            let value = Value::deserialize(deserializer)?;
            Ok(BlockHeaderExtra::new(value.0))
        }
    }
}

impl Serialize for BlockHeaderExtra {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            format!("0x{}", hex::encode(self.0)).serialize(serializer)
        } else {
            serializer.serialize_newtype_struct("BlockHeaderExtra", &self.0)
        }
    }
}

#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize, JsonSchema,
)]
pub struct BlockIdAndNumber {
    pub id: HashValue,
    pub number: BlockNumber,
}

impl BlockIdAndNumber {
    pub fn new(id: HashValue, number: BlockNumber) -> Self {
        Self { id, number }
    }
    pub fn id(&self) -> HashValue {
        self.id
    }
    pub fn number(&self) -> BlockNumber {
        self.number
    }
}

impl From<BlockHeader> for BlockIdAndNumber {
    fn from(header: BlockHeader) -> Self {
        Self {
            id: header.id(),
            number: header.number(),
        }
    }
}

/// block timestamp allowed future times
pub const ALLOWED_FUTURE_BLOCKTIME: u64 = 30000; // 30 second;

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, CryptoHasher, CryptoHash, JsonSchema)]
pub struct BlockHeader {
    #[serde(skip)]
    id: Option<HashValue>,
    /// Parent hash.
    parent_hash: HashValue,
    /// Block timestamp.
    timestamp: u64,
    /// Block number.
    number: BlockNumber,
    /// Block author.
    author: AccountAddress,
    /// Block author auth key.
    /// this field is deprecated
    author_auth_key: Option<AuthenticationKey>,
    /// The transaction accumulator root hash after executing this block.
    txn_accumulator_root: HashValue,
    /// The parent block info's block accumulator root hash.
    block_accumulator_root: HashValue,
    /// The last transaction state_root of this block after execute.
    state_root: HashValue,
    /// Gas used for contracts execution.
    gas_used: u64,
    /// Block difficulty
    #[schemars(with = "String")]
    difficulty: U256,
    /// hash for block body
    body_hash: HashValue,
    /// The chain id
    chain_id: ChainId,
    /// Consensus nonce field.
    nonce: u32,
    /// block header extra
    extra: BlockHeaderExtra,
}

impl BlockHeader {
    pub fn new(
        parent_hash: HashValue,
        timestamp: u64,
        number: BlockNumber,
        author: AccountAddress,
        txn_accumulator_root: HashValue,
        block_accumulator_root: HashValue,
        state_root: HashValue,
        gas_used: u64,
        difficulty: U256,
        body_hash: HashValue,
        chain_id: ChainId,
        nonce: u32,
        extra: BlockHeaderExtra,
    ) -> BlockHeader {
        Self::new_with_auth_key(
            parent_hash,
            timestamp,
            number,
            author,
            None,
            txn_accumulator_root,
            block_accumulator_root,
            state_root,
            gas_used,
            difficulty,
            body_hash,
            chain_id,
            nonce,
            extra,
        )
    }

    // the author_auth_key field is deprecated, but keep this fn for compat with old block.
    fn new_with_auth_key(
        parent_hash: HashValue,
        timestamp: u64,
        number: BlockNumber,
        author: AccountAddress,
        author_auth_key: Option<AuthenticationKey>,
        txn_accumulator_root: HashValue,
        block_accumulator_root: HashValue,
        state_root: HashValue,
        gas_used: u64,
        difficulty: U256,
        body_hash: HashValue,
        chain_id: ChainId,
        nonce: u32,
        extra: BlockHeaderExtra,
    ) -> BlockHeader {
        let mut header = BlockHeader {
            id: None,
            parent_hash,
            block_accumulator_root,
            number,
            timestamp,
            author,
            author_auth_key,
            txn_accumulator_root,
            state_root,
            gas_used,
            difficulty,
            nonce,
            body_hash,
            chain_id,
            extra,
        };
        header.id = Some(header.crypto_hash());
        header
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
        self.id
            .expect("BlockHeader id should bean Some after init.")
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

    pub fn author_auth_key(&self) -> Option<AuthenticationKey> {
        self.author_auth_key
    }

    pub fn txn_accumulator_root(&self) -> HashValue {
        self.txn_accumulator_root
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

    pub fn block_accumulator_root(&self) -> HashValue {
        self.block_accumulator_root
    }

    pub fn body_hash(&self) -> HashValue {
        self.body_hash
    }

    pub fn chain_id(&self) -> ChainId {
        self.chain_id
    }

    pub fn extra(&self) -> &BlockHeaderExtra {
        &self.extra
    }

    pub fn is_genesis(&self) -> bool {
        self.number == 0
    }

    pub fn genesis_block_header(
        parent_hash: HashValue,
        timestamp: u64,
        txn_accumulator_root: HashValue,
        state_root: HashValue,
        difficulty: U256,
        body_hash: HashValue,
        chain_id: ChainId,
    ) -> Self {
        Self::new(
            parent_hash,
            timestamp,
            0,
            CORE_CODE_ADDRESS,
            txn_accumulator_root,
            *ACCUMULATOR_PLACEHOLDER_HASH,
            state_root,
            0,
            difficulty,
            body_hash,
            chain_id,
            0,
            BlockHeaderExtra::default(),
        )
    }

    pub fn random() -> Self {
        Self::new(
            HashValue::random(),
            rand::random(),
            rand::random(),
            AccountAddress::random(),
            HashValue::random(),
            HashValue::random(),
            HashValue::random(),
            rand::random(),
            U256::max_value(),
            HashValue::random(),
            ChainId::test(),
            0,
            BlockHeaderExtra([0u8; 4]),
        )
    }

    pub fn as_builder(&self) -> BlockHeaderBuilder {
        BlockHeaderBuilder::new_with(self.clone())
    }
}

impl<'de> Deserialize<'de> for BlockHeader {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(rename = "BlockHeader")]
        struct BlockHeaderData {
            parent_hash: HashValue,
            timestamp: u64,
            number: BlockNumber,
            author: AccountAddress,
            author_auth_key: Option<AuthenticationKey>,
            txn_accumulator_root: HashValue,
            block_accumulator_root: HashValue,
            state_root: HashValue,
            gas_used: u64,
            difficulty: U256,
            body_hash: HashValue,
            chain_id: ChainId,
            nonce: u32,
            extra: BlockHeaderExtra,
        }

        let header_data = BlockHeaderData::deserialize(deserializer)?;
        let block_header = Self::new_with_auth_key(
            header_data.parent_hash,
            header_data.timestamp,
            header_data.number,
            header_data.author,
            header_data.author_auth_key,
            header_data.txn_accumulator_root,
            header_data.block_accumulator_root,
            header_data.state_root,
            header_data.gas_used,
            header_data.difficulty,
            header_data.body_hash,
            header_data.chain_id,
            header_data.nonce,
            header_data.extra,
        );
        Ok(block_header)
    }
}

impl Default for BlockHeader {
    fn default() -> Self {
        Self::new(
            HashValue::zero(),
            0,
            0,
            AccountAddress::ZERO,
            HashValue::zero(),
            HashValue::zero(),
            HashValue::zero(),
            0,
            0.into(),
            HashValue::zero(),
            ChainId::test(),
            0,
            BlockHeaderExtra([0u8; 4]),
        )
    }
}

impl Sample for BlockHeader {
    fn sample() -> Self {
        Self::new(
            HashValue::zero(),
            1610110515000,
            0,
            genesis_address(),
            *ACCUMULATOR_PLACEHOLDER_HASH,
            *ACCUMULATOR_PLACEHOLDER_HASH,
            *SPARSE_MERKLE_PLACEHOLDER_HASH,
            0,
            U256::from(1),
            BlockBody::sample().crypto_hash(),
            ChainId::test(),
            0,
            BlockHeaderExtra([0u8; 4]),
        )
    }
}

#[allow(clippy::from_over_into)]
impl Into<RawBlockHeader> for BlockHeader {
    fn into(self) -> RawBlockHeader {
        RawBlockHeader {
            parent_hash: self.parent_hash,
            timestamp: self.timestamp,
            number: self.number,
            author: self.author,
            author_auth_key: self.author_auth_key,
            accumulator_root: self.txn_accumulator_root,
            parent_block_accumulator_root: self.block_accumulator_root,
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
    /// this field is deprecated
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

#[derive(Default)]
pub struct BlockHeaderBuilder {
    buffer: BlockHeader,
}

impl BlockHeaderBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn random() -> Self {
        Self {
            buffer: BlockHeader::random(),
        }
    }

    fn new_with(buffer: BlockHeader) -> Self {
        Self { buffer }
    }

    pub fn with_parent_hash(mut self, parent_hash: HashValue) -> Self {
        self.buffer.parent_hash = parent_hash;
        self
    }

    pub fn with_timestamp(mut self, timestamp: u64) -> Self {
        self.buffer.timestamp = timestamp;
        self
    }

    pub fn with_number(mut self, number: BlockNumber) -> Self {
        self.buffer.number = number;
        self
    }

    pub fn with_author(mut self, author: AccountAddress) -> Self {
        self.buffer.author = author;
        self
    }

    pub fn with_author_auth_key(mut self, author_auth_key: Option<AuthenticationKey>) -> Self {
        self.buffer.author_auth_key = author_auth_key;
        self
    }

    pub fn with_accumulator_root(mut self, accumulator_root: HashValue) -> Self {
        self.buffer.txn_accumulator_root = accumulator_root;
        self
    }

    pub fn with_parent_block_accumulator_root(
        mut self,
        parent_block_accumulator_root: HashValue,
    ) -> Self {
        self.buffer.block_accumulator_root = parent_block_accumulator_root;
        self
    }

    pub fn with_state_root(mut self, state_root: HashValue) -> Self {
        self.buffer.state_root = state_root;
        self
    }

    pub fn with_gas_used(mut self, gas_used: u64) -> Self {
        self.buffer.gas_used = gas_used;
        self
    }

    pub fn with_difficulty(mut self, difficulty: U256) -> Self {
        self.buffer.difficulty = difficulty;
        self
    }

    pub fn with_body_hash(mut self, body_hash: HashValue) -> Self {
        self.buffer.body_hash = body_hash;
        self
    }

    pub fn with_chain_id(mut self, chain_id: ChainId) -> Self {
        self.buffer.chain_id = chain_id;
        self
    }

    pub fn with_nonce(mut self, nonce: u32) -> Self {
        self.buffer.nonce = nonce;
        self
    }

    pub fn with_extra(mut self, extra: BlockHeaderExtra) -> Self {
        self.buffer.extra = extra;
        self
    }

    pub fn build(mut self) -> BlockHeader {
        self.buffer.id = Some(self.buffer.crypto_hash());
        self.buffer
    }
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

#[allow(clippy::from_over_into)]
impl Into<BlockBody> for Vec<SignedUserTransaction> {
    fn into(self) -> BlockBody {
        BlockBody {
            transactions: self,
            uncles: None,
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<Vec<SignedUserTransaction>> for BlockBody {
    fn into(self) -> Vec<SignedUserTransaction> {
        self.transactions
    }
}

impl Sample for BlockBody {
    fn sample() -> Self {
        Self {
            transactions: vec![],
            uncles: None,
        }
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

    pub fn uncle_ids(&self) -> Vec<HashValue> {
        self.uncles()
            .map(|uncles| uncles.iter().map(|header| header.id()).collect())
            .unwrap_or_default()
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

impl Sample for Block {
    fn sample() -> Self {
        Self {
            header: BlockHeader::sample(),
            body: BlockBody::sample(),
        }
    }
}

/// `BlockInfo` is the object we store in the storage. It consists of the
/// block as well as the execution result of this block.
#[derive(
    Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, CryptoHasher, CryptoHash, JsonSchema,
)]
pub struct BlockInfo {
    /// Block id
    pub block_id: HashValue,
    /// The total difficulty.
    #[schemars(with = "String")]
    pub total_difficulty: U256,
    /// The transaction accumulator info
    pub txn_accumulator_info: AccumulatorInfo,
    /// The block accumulator info.
    pub block_accumulator_info: AccumulatorInfo,
}

impl BlockInfo {
    pub fn new(
        block_id: HashValue,
        total_difficulty: U256,
        txn_accumulator_info: AccumulatorInfo,
        block_accumulator_info: AccumulatorInfo,
    ) -> Self {
        Self {
            block_id,
            total_difficulty,
            txn_accumulator_info,
            block_accumulator_info,
        }
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

impl Sample for BlockInfo {
    fn sample() -> Self {
        Self {
            block_id: BlockHeader::sample().id(),
            total_difficulty: 0.into(),
            txn_accumulator_info: AccumulatorInfo::sample(),
            block_accumulator_info: AccumulatorInfo::sample(),
        }
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
    /// The transaction accumulator root hash after executing this block.
    pub txn_accumulator_root: HashValue,
    /// The block accumulator root hash.
    pub block_accumulator_root: HashValue,
    /// The last transaction state_root of this block after execute.
    pub state_root: HashValue,
    /// Gas used for contracts execution.
    pub gas_used: u64,
    /// hash for block body
    pub body_hash: HashValue,
    /// body of the block
    pub body: BlockBody,
    /// The chain id
    pub chain_id: ChainId,
    /// Block difficulty
    pub difficulty: U256,
    /// Block consensus strategy
    pub strategy: ConsensusStrategy,
}

impl BlockTemplate {
    pub fn new(
        parent_block_accumulator_root: HashValue,
        accumulator_root: HashValue,
        state_root: HashValue,
        gas_used: u64,
        body: BlockBody,
        chain_id: ChainId,
        difficulty: U256,
        strategy: ConsensusStrategy,
        block_metadata: BlockMetadata,
    ) -> Self {
        let (parent_hash, timestamp, author, _author_auth_key, _, number, _, _) =
            block_metadata.into_inner();
        Self {
            parent_hash,
            block_accumulator_root: parent_block_accumulator_root,
            timestamp,
            number,
            author,
            txn_accumulator_root: accumulator_root,
            state_root,
            gas_used,
            body_hash: body.hash(),
            body,
            chain_id,
            difficulty,
            strategy,
        }
    }

    pub fn into_block(self, nonce: u32, extra: BlockHeaderExtra) -> Block {
        let header = BlockHeader::new(
            self.parent_hash,
            self.timestamp,
            self.number,
            self.author,
            self.txn_accumulator_root,
            self.block_accumulator_root,
            self.state_root,
            self.gas_used,
            self.difficulty,
            self.body_hash,
            self.chain_id,
            nonce,
            extra,
        );
        Block {
            header,
            body: self.body,
        }
    }

    pub fn as_raw_block_header(&self) -> RawBlockHeader {
        RawBlockHeader {
            parent_hash: self.parent_hash,
            timestamp: self.timestamp,
            number: self.number,
            author: self.author,
            author_auth_key: None,
            accumulator_root: self.txn_accumulator_root,
            parent_block_accumulator_root: self.block_accumulator_root,
            state_root: self.state_root,
            gas_used: self.gas_used,
            body_hash: self.body_hash,
            difficulty: self.difficulty,
            chain_id: self.chain_id,
        }
    }

    pub fn as_pow_header_blob(&self) -> Vec<u8> {
        let mut blob = Vec::new();
        let raw_header = self.as_raw_block_header();
        let raw_header_hash = raw_header.crypto_hash();
        let mut dh = [0u8; 32];
        raw_header.difficulty.to_big_endian(&mut dh);
        let extend_and_nonce = [0u8; 12];

        blob.extend_from_slice(raw_header_hash.to_vec().as_slice());
        blob.extend_from_slice(&extend_and_nonce);
        blob.extend_from_slice(&dh);
        blob
    }

    pub fn into_block_header(self, nonce: u32, extra: BlockHeaderExtra) -> BlockHeader {
        BlockHeader::new(
            self.parent_hash,
            self.timestamp,
            self.number,
            self.author,
            self.txn_accumulator_root,
            self.block_accumulator_root,
            self.state_root,
            self.gas_used,
            self.difficulty,
            self.body_hash,
            self.chain_id,
            nonce,
            extra,
        )
    }
}

#[derive(Clone, Debug, Hash, Serialize, Deserialize, CryptoHasher, CryptoHash)]
pub struct ExecutedBlock {
    pub block: Block,
    pub block_info: BlockInfo,
}

impl ExecutedBlock {
    pub fn new(block: Block, block_info: BlockInfo) -> Self {
        ExecutedBlock { block, block_info }
    }

    pub fn total_difficulty(&self) -> U256 {
        self.block_info.total_difficulty
    }

    pub fn block(&self) -> &Block {
        &self.block
    }

    pub fn block_info(&self) -> &BlockInfo {
        &self.block_info
    }

    pub fn header(&self) -> &BlockHeader {
        self.block.header()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlockSummary {
    pub block_header: BlockHeader,
    pub uncles: Vec<BlockHeader>,
}

impl BlockSummary {
    pub fn uncles(&self) -> &[BlockHeader] {
        &self.uncles
    }

    pub fn header(&self) -> &BlockHeader {
        &self.block_header
    }
}

impl From<Block> for BlockSummary {
    fn from(block: Block) -> Self {
        Self {
            block_header: block.header,
            uncles: block.body.uncles.unwrap_or_default(),
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<(BlockHeader, Vec<BlockHeader>)> for BlockSummary {
    fn into(self) -> (BlockHeader, Vec<BlockHeader>) {
        (self.block_header, self.uncles)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UncleSummary {
    /// total uncle
    pub uncles: u64,
    /// sum(number of the block which contain uncle block - uncle parent block number).
    pub sum: u64,
    pub avg: u64,
    pub time_sum: u64,
    pub time_avg: u64,
}

impl UncleSummary {
    pub fn new(uncles: u64, sum: u64, time_sum: u64) -> Self {
        let (avg, time_avg) = (
            sum.checked_div(uncles).unwrap_or_default(),
            time_sum.checked_div(uncles).unwrap_or_default(),
        );
        Self {
            uncles,
            sum,
            avg,
            time_sum,
            time_avg,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EpochUncleSummary {
    /// epoch number
    pub epoch: u64,
    pub number_summary: UncleSummary,
    pub epoch_summary: UncleSummary,
}

impl EpochUncleSummary {
    pub fn new(epoch: u64, number_summary: UncleSummary, epoch_summary: UncleSummary) -> Self {
        Self {
            epoch,
            number_summary,
            epoch_summary,
        }
    }
}
