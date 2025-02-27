// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod legacy;
#[cfg(test)]
mod tests;

mod block_header_data;
pub mod raw_block_header;

use crate::account_address::AccountAddress;
use crate::block_metadata::BlockMetadata;
use crate::genesis_config::{ChainId, ConsensusStrategy};
use crate::language_storage::CORE_CODE_ADDRESS;
use crate::transaction::SignedUserTransaction;
use crate::U256;
use bcs_ext::Sample;
use block_header_data::{BlockHeaderDataInVega, BlockHeaderDataLatest};
use lazy_static::lazy_static;
pub use legacy::{
    Block as LegacyBlock, BlockBody as LegacyBlockBody, BlockHeader as LegacyBlockHeader,
};
use raw_block_header::RawBlockHeader;
use schemars::{self, JsonSchema};
use serde::de::{self, Error, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
pub use starcoin_accumulator::accumulator_info::AccumulatorInfo;
use starcoin_crypto::hash::{ACCUMULATOR_PLACEHOLDER_HASH, SPARSE_MERKLE_PLACEHOLDER_HASH};
use starcoin_crypto::{
    hash::{CryptoHash, CryptoHasher, PlainCryptoHash},
    HashValue,
};
use starcoin_vm_types::account_config::genesis_address;
use starcoin_vm_types::transaction::authenticator::AuthenticationKey;
use std::fmt::{self, Formatter};
use std::hash::Hash;
use std::sync::Mutex;

/// Type for block number.
pub type BlockNumber = u64;

pub type ParentsHash = Vec<HashValue>;

pub type Version = u32;

pub const BLOCK_HEADER_VERSION_1: BlockNumber = 1024;

lazy_static! {
    static ref TEST_FLEXIDAG_FORK_HEIGHT: Mutex<BlockNumber> = Mutex::new(10000);
    static ref CUSTOM_FLEXIDAG_FORK_HEIGHT: Mutex<BlockNumber> = Mutex::new(10000);
}

pub fn get_test_flexidag_fork_height() -> BlockNumber {
    *TEST_FLEXIDAG_FORK_HEIGHT.lock().unwrap()
}

pub fn get_custom_flexidag_fork_height() -> BlockNumber {
    *CUSTOM_FLEXIDAG_FORK_HEIGHT.lock().unwrap()
}

pub fn set_customm_flexidag_fork_height(value: BlockNumber) {
    let mut num = TEST_FLEXIDAG_FORK_HEIGHT.lock().unwrap();
    *num = value;
}

pub fn reset_test_custom_fork_height() {
    *TEST_FLEXIDAG_FORK_HEIGHT.lock().unwrap() = 10000;
    *CUSTOM_FLEXIDAG_FORK_HEIGHT.lock().unwrap() = 10000;
}

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
            Ok(Self::new(extra))
        } else {
            #[derive(::serde::Deserialize)]
            #[serde(rename = "BlockHeaderExtra")]
            struct Value([u8; 4]);
            let value = Value::deserialize(deserializer)?;
            Ok(Self::new(value.0))
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

#[derive(Clone, Debug, Hash, Eq, PartialEq, CryptoHasher, CryptoHash, JsonSchema)]
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
    /// Parents hash.
    parents_hash: ParentsHash,
    /// Header version
    version: Version,
    /// pruning point
    pruning_point: HashValue,
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
        parents_hash: ParentsHash,
        version: Version,
        pruning_point: HashValue,
    ) -> Self {
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
            parents_hash,
            version,
            pruning_point,
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
        parents_hash: ParentsHash,
        version: Version,
        pruning_point: HashValue,
    ) -> Self {
        let header = BlockHeaderDataLatest {
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
            parents_hash: Some(parents_hash),
            version,
            pruning_point,
        };
        let mut result = Self {
            id: None,
            parent_hash: header.parent_hash,
            timestamp: header.timestamp,
            number: header.number,
            author: header.author,
            author_auth_key: header.author_auth_key,
            txn_accumulator_root: header.txn_accumulator_root,
            block_accumulator_root: header.block_accumulator_root,
            state_root: header.state_root,
            gas_used: header.gas_used,
            difficulty: header.difficulty,
            body_hash: header.body_hash,
            chain_id: header.chain_id,
            nonce: header.nonce,
            extra: header.extra,
            parents_hash: header
                .parents_hash
                .clone()
                .expect("parents hash should not be none, use [] instead if it is"),
            version: header.version,
            pruning_point: header.pruning_point,
        };
        let id = Some(header.into_hash());
        result.id = id;
        result
    }

    pub fn as_pow_header_blob(&self) -> Vec<u8> {
        let mut blob = Vec::new();
        let raw_header: RawBlockHeader = self.to_owned().into();
        let raw_header_hash = raw_header.calc_hash();
        let mut diff = [0u8; 32];
        raw_header.difficulty.to_big_endian(&mut diff);
        let extend_and_nonce = [0u8; 12];
        blob.extend_from_slice(raw_header_hash.to_vec().as_slice());
        blob.extend_from_slice(&extend_and_nonce);
        blob.extend_from_slice(&diff);
        blob
    }

    pub fn id(&self) -> HashValue {
        self.id.expect("BlockHeader id should be Some after init.")
    }

    pub fn parent_hash(&self) -> HashValue {
        self.parent_hash
    }

    pub fn parents_hash(&self) -> ParentsHash {
        self.parents_hash.clone()
    }

    pub fn version(&self) -> Version {
        self.version
    }

    pub fn pruning_point(&self) -> HashValue {
        self.pruning_point
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
            vec![], // in fact, it is better to put the [origin] into this field but here [] is done for the adaptability.
            0,
            HashValue::zero(),
        )
    }

    //for test
    pub fn dag_genesis_random(dag_genesis_number: BlockNumber) -> Self {
        let mut header = Self::random();
        header.parents_hash = vec![];
        header.number = dag_genesis_number;
        header.id = Some(header.calc_hash());
        header
    }

    //for test
    pub fn dag_genesis_random_with_parent(parent: Self) -> anyhow::Result<Self> {
        let header_builder = BlockHeaderBuilder::random();
        anyhow::Result::Ok(
            header_builder
                .with_parent_hash(parent.id())
                .with_parents_hash(vec![parent.id()])
                .with_number(0)
                .build(),
        )
    }

    pub fn random() -> Self {
        Self::new(
            HashValue::random(),
            rand::random(),
            rand::random::<u64>(),
            AccountAddress::random(),
            HashValue::random(),
            HashValue::random(),
            HashValue::random(),
            rand::random(),
            rand::random::<u64>().into(),
            HashValue::random(),
            ChainId::test(),
            0,
            BlockHeaderExtra([0u8; 4]),
            vec![HashValue::random(), HashValue::random()],
            rand::random::<Version>(),
            HashValue::random(),
        )
    }

    pub fn rational_random(body_hash: HashValue) -> Self {
        Self::new(
            HashValue::random(),
            rand::random(),
            rand::random::<u64>(),
            AccountAddress::random(),
            HashValue::random(),
            HashValue::random(),
            HashValue::random(),
            rand::random(),
            rand::random::<u64>().into(),
            body_hash,
            ChainId::test(),
            0,
            BlockHeaderExtra([0u8; 4]),
            vec![HashValue::random(), HashValue::random()],
            rand::random::<Version>(),
            HashValue::random(),
        )
    }

    pub fn calc_hash(&self) -> HashValue {
        let latest_data: BlockHeaderDataLatest = self.clone().into();
        latest_data.into_hash()
    }

    pub fn as_builder(&self) -> BlockHeaderBuilder {
        BlockHeaderBuilder::new_with(self.clone())
    }

    fn upgrade(&self) -> bool {
        Self::check_upgrade(self.number(), self.chain_id())
    }

    pub fn check_upgrade(number: BlockNumber, chain_id: ChainId) -> bool {
        if number == 0 {
            false
        } else if chain_id.is_vega() {
            number >= 3300000
        } else if chain_id.is_halley() {
            number >= 3100000
        } else if chain_id.is_proxima() {
            number >= 200
        } else {
            true
        }
    }
}

impl Serialize for BlockHeader {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if !self.upgrade() {
            // for vega no pruning point
            let header_data: BlockHeaderDataInVega = self.clone().into();
            header_data.serialize(serializer)
        } else {
            let header_data: BlockHeaderDataLatest = self.clone().into();
            header_data.serialize(serializer)
        }
    }
}

impl<'de> Deserialize<'de> for BlockHeader {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        struct BlockHeaderVisitor;

        impl<'de> Visitor<'de> for BlockHeaderVisitor {
            type Value = BlockHeader;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct BlockHeader")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<BlockHeader, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let parent_hash: HashValue = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let timestamp: u64 = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let number: BlockNumber = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let author: AccountAddress = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let author_auth_key: Option<AuthenticationKey> = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let txn_accumulator_root: HashValue = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let block_accumulator_root: HashValue = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let state_root: HashValue = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let gas_used: u64 = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let difficulty: U256 = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let body_hash: HashValue = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let chain_id: ChainId = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let nonce: u32 = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let extra: BlockHeaderExtra = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let parents_hash: Option<ParentsHash> =
                    seq.next_element().map_or(Some(vec![]), |value| {
                        value.map_or(Some(vec![]), |value| value)
                    });

                let (version, pruning_point) = if !BlockHeader::check_upgrade(number, chain_id) {
                    (0, HashValue::zero())
                } else {
                    let version: Version = seq
                        .next_element()?
                        .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                    let pruning_point: HashValue =
                        seq.next_element().map_or(HashValue::zero(), |value| {
                            value.map_or(HashValue::zero(), |value| value)
                        });
                    (version, pruning_point)
                };

                let header = BlockHeader::new_with_auth_key(
                    parent_hash,
                    timestamp,
                    number,
                    author,
                    author_auth_key,
                    txn_accumulator_root,
                    block_accumulator_root,
                    state_root,
                    gas_used,
                    difficulty,
                    body_hash,
                    chain_id,
                    nonce,
                    extra,
                    parents_hash.map_or(vec![], |value| value),
                    version,
                    pruning_point,
                );
                Ok(header)
            }
        }

        const BLOCK_HEADER_FIELDS: &[&str] = &[
            "parent_hash",
            "timestamp",
            "number",
            "author",
            "author_auth_key",
            "txn_accumulator_root",
            "block_accumulator_root",
            "state_root",
            "gas_used",
            "difficulty",
            "body_hash",
            "chain_id",
            "nonce",
            "extra",
            "parents_hash",
            "version",
            "pruning_point",
        ];

        deserializer.deserialize_struct("BlockHeader", BLOCK_HEADER_FIELDS, BlockHeaderVisitor)
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
            vec![],
            0,
            HashValue::zero(),
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
            vec![],
            0,
            HashValue::zero(),
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
            parents_hash: self.parents_hash,
            version: self.version,
            pruning_point: self.pruning_point,
        }
    }
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
    pub fn with_parents_hash(mut self, parent_hash: ParentsHash) -> Self {
        self.buffer.parents_hash = parent_hash;
        self
    }

    pub fn with_pruning_point(mut self, pruning_point: HashValue) -> Self {
        self.buffer.pruning_point = pruning_point;
        self
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

    pub fn with_version(mut self, version: Version) -> Self {
        self.buffer.version = version;
        self
    }

    pub fn build(self) -> BlockHeader {
        let crypto_data: BlockHeaderDataLatest = self.into();
        let mut header = BlockHeader {
            id: None,
            parent_hash: crypto_data.parent_hash,
            timestamp: crypto_data.timestamp,
            number: crypto_data.number,
            author: crypto_data.author,
            author_auth_key: crypto_data.author_auth_key,
            txn_accumulator_root: crypto_data.txn_accumulator_root,
            block_accumulator_root: crypto_data.block_accumulator_root,
            state_root: crypto_data.state_root,
            gas_used: crypto_data.gas_used,
            difficulty: crypto_data.difficulty,
            body_hash: crypto_data.body_hash,
            chain_id: crypto_data.chain_id,
            nonce: crypto_data.nonce,
            extra: crypto_data.extra,
            parents_hash: crypto_data
                .parents_hash
                .clone()
                .expect("parents hash should not be none, use [] instead if it is"),
            version: crypto_data.version,
            pruning_point: crypto_data.pruning_point,
        };
        let id = Some(crypto_data.into_hash());
        header.id = id;
        header
    }
}

#[derive(Default, Clone, Debug, Hash, Eq, PartialEq, Serialize, CryptoHasher, CryptoHash)]
pub struct BlockBody {
    /// The transactions in this block.
    pub transactions: Vec<SignedUserTransaction>,
    /// uncles block header
    pub uncles: Option<Vec<BlockHeader>>,
}

impl<'de> Deserialize<'de> for BlockBody {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct BlockBodyVisitor;

        impl<'de> Visitor<'de> for BlockBodyVisitor {
            type Value = BlockBody;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct BlockBody")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let transactions: Vec<SignedUserTransaction> = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let uncles: Option<Vec<BlockHeader>> = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;

                Ok(BlockBody {
                    transactions,
                    uncles,
                })
            }
        }

        const BLOCK_BODY_FIELDS: &[&str] = &["transactions", "uncles"];

        deserializer.deserialize_struct("BlockBody", BLOCK_BODY_FIELDS, BlockBodyVisitor)
    }
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
    pub fn new_empty() -> Self {
        Self {
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
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, CryptoHasher, CryptoHash)]
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
        Self {
            header,
            body: body.into(),
        }
    }

    pub fn parent_hash(&self) -> HashValue {
        self.header.parent_hash()
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

        BlockMetadata::new_with_parents(
            self.header.parent_hash(),
            self.header.timestamp,
            self.header.author,
            self.header.author_auth_key,
            uncles,
            self.header.number,
            self.header.chain_id,
            parent_gas_used,
            self.header.parents_hash.clone(),
        )
    }

    pub fn random() -> Self {
        let body = BlockBody::sample();
        let header = BlockHeader::random();
        Self { header, body }
    }

    pub fn rational_random() -> Self {
        let uncle1 = crate::block::BlockHeaderBuilder::new()
            .with_chain_id(ChainId::vega())
            .with_number(512)
            .with_parent_hash(HashValue::random())
            .with_parents_hash(vec![
                HashValue::random(),
                HashValue::random(),
                HashValue::random(),
            ])
            .build();

        let uncle2 = crate::block::BlockHeaderBuilder::new()
            .with_number(128)
            .with_chain_id(ChainId::vega())
            .with_parent_hash(HashValue::random())
            .with_parents_hash(vec![
                HashValue::random(),
                HashValue::random(),
                HashValue::random(),
            ])
            .build();
        let body = crate::block::BlockBody {
            transactions: vec![
                SignedUserTransaction::sample(),
                SignedUserTransaction::sample(),
                SignedUserTransaction::sample(),
            ],
            uncles: Some(vec![uncle1, uncle2]),
        };

        let header = crate::block::BlockHeaderBuilder::new()
            .with_number(1024)
            .with_chain_id(ChainId::vega())
            .with_parent_hash(HashValue::random())
            .with_parents_hash(vec![
                HashValue::random(),
                HashValue::random(),
                HashValue::random(),
            ])
            .with_body_hash(body.hash())
            .build();

        Self { header, body }
    }
}

impl<'de> Deserialize<'de> for Block {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct BlockVisitor;

        impl<'de> Visitor<'de> for BlockVisitor {
            type Value = Block;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Block")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let header: BlockHeader = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let body: BlockBody = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;

                if body.hash() != header.body_hash {
                    return std::result::Result::Err(serde::de::Error::custom(
                        "Block body hash does not match header body hash",
                    ));
                }

                Ok(Block { header, body })
            }
        }

        const BLOCK_FIELDS: &[&str] = &["header", "body"];

        deserializer.deserialize_struct("Block", BLOCK_FIELDS, BlockVisitor)
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
    /// parents
    pub parents_hash: ParentsHash,
    /// version
    pub version: Version,
    /// pruning point
    pub pruning_point: HashValue,
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
        version: Version,
        pruning_point: HashValue,
    ) -> Self {
        let (parent_hash, timestamp, author, _author_auth_key, _, number, _, _, parents_hash) =
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
            // for an upgraded binary, parents_hash should never be None.
            parents_hash,
            version,
            pruning_point,
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
            self.parents_hash,
            self.version,
            self.pruning_point,
        );

        Block {
            header,
            body: self.body,
        }
    }

    fn as_raw_block_header(&self) -> RawBlockHeader {
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
            parents_hash: self.parents_hash.clone(),
            version: self.version,
            pruning_point: self.pruning_point,
        }
    }

    pub fn as_pow_header_blob(&self) -> Vec<u8> {
        let mut blob = Vec::new();
        let raw_header: RawBlockHeader = self.as_raw_block_header();
        let raw_header_hash = raw_header.calc_hash();
        let mut dh = [0u8; 32];
        raw_header.difficulty.to_big_endian(&mut dh);
        let extend_and_nonce = [0u8; 12];
        blob.extend_from_slice(raw_header_hash.to_vec().as_slice());
        blob.extend_from_slice(&extend_and_nonce);
        blob.extend_from_slice(&dh);

        blob
    }
}

#[derive(Clone, Debug, Hash, Serialize, Deserialize, CryptoHasher, CryptoHash)]
pub struct ExecutedBlock {
    pub block: Block,
    pub block_info: BlockInfo,
}

impl ExecutedBlock {
    pub fn new(block: Block, block_info: BlockInfo) -> Self {
        Self { block, block_info }
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
