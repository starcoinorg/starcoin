use super::{AccountAddress, BlockHeaderExtra, BlockNumber, ChainId, SignedUserTransaction, U256};
use schemars::{self, JsonSchema};
use serde::{Deserialize, Deserializer, Serialize};
use starcoin_crypto::{
    hash::{CryptoHash, CryptoHasher, PlainCryptoHash},
    HashValue,
};
use starcoin_vm_types::transaction::authenticator::AuthenticationKey;

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, CryptoHasher, CryptoHash, JsonSchema)]
#[serde(rename = "BlockHeader")]
pub struct BlockHeader {
    #[serde(skip)]
    pub id: Option<HashValue>,
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
    // the author_auth_key field is deprecated, but keep this fn for compat with old block.
    pub(crate) fn new_with_auth_key(
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

    pub fn number(&self) -> BlockNumber {
        self.number
    }

    pub fn id(&self) -> HashValue {
        self.id.unwrap()
    }
}

impl From<crate::block::BlockHeader> for BlockHeader {
    fn from(v: crate::block::BlockHeader) -> Self {
        Self {
            id: v.id,
            parent_hash: v.parent_hash,
            timestamp: v.timestamp,
            number: v.number,
            author: v.author,
            author_auth_key: v.author_auth_key,
            txn_accumulator_root: v.txn_accumulator_root,
            block_accumulator_root: v.block_accumulator_root,
            state_root: v.state_root,
            gas_used: v.gas_used,
            difficulty: v.difficulty,
            body_hash: v.body_hash,
            chain_id: v.chain_id,
            nonce: v.nonce,
            extra: v.extra,
        }
    }
}

impl From<BlockHeader> for crate::block::BlockHeader {
    fn from(v: BlockHeader) -> Self {
        let id = v.id.or_else(|| Some(v.crypto_hash()));
        Self {
            id,
            parent_hash: v.parent_hash,
            timestamp: v.timestamp,
            number: v.number,
            author: v.author,
            author_auth_key: v.author_auth_key,
            txn_accumulator_root: v.txn_accumulator_root,
            block_accumulator_root: v.block_accumulator_root,
            state_root: v.state_root,
            gas_used: v.gas_used,
            difficulty: v.difficulty,
            body_hash: v.body_hash,
            chain_id: v.chain_id,
            nonce: v.nonce,
            extra: v.extra,
            parents_hash: None,
        }
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
    pub fn hash(&self) -> HashValue {
        self.crypto_hash()
    }
}

impl From<BlockBody> for crate::block::BlockBody {
    fn from(value: BlockBody) -> Self {
        let BlockBody {
            transactions,
            uncles,
        } = value;

        Self {
            transactions,
            uncles: uncles.map(|u| u.into_iter().map(Into::into).collect()),
        }
    }
}

impl From<crate::block::BlockBody> for BlockBody {
    fn from(value: crate::block::BlockBody) -> Self {
        let crate::block::BlockBody {
            transactions,
            uncles,
        } = value;

        Self {
            transactions,
            uncles: uncles.map(|u| u.into_iter().map(Into::into).collect()),
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
    pub fn id(&self) -> HashValue {
        self.header.id()
    }

    pub fn header(&self) -> &BlockHeader {
        &self.header
    }
}

impl From<Block> for crate::block::Block {
    fn from(value: Block) -> Self {
        Self {
            header: value.header.into(),
            body: value.body.into(),
        }
    }
}

impl From<crate::block::Block> for Block {
    fn from(value: crate::block::Block) -> Self {
        Self {
            header: value.header.into(),
            body: value.body.into(),
        }
    }
}
