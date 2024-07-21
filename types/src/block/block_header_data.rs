use schemars::{self, JsonSchema};
use serde::{Deserialize, Serialize};
use starcoin_crypto::{
    hash::{CryptoHash, CryptoHasher, PlainCryptoHash},
    HashValue,
};
use starcoin_vm_types::{
    account_address::AccountAddress, genesis_config::ChainId,
    transaction::authenticator::AuthenticationKey,
};

use super::{BlockHeader, BlockHeaderBuilder, BlockHeaderExtra, BlockNumber, ParentsHash, Version};
use crate::U256;

// for calculating the block id
#[derive(Debug, Serialize, Deserialize, CryptoHasher, CryptoHash)]
#[serde(rename = "BlockHeader")]
pub(crate) struct BlockHeaderDataLatest {
    pub parent_hash: HashValue,
    pub timestamp: u64,
    pub number: BlockNumber,
    pub author: AccountAddress,
    pub author_auth_key: Option<AuthenticationKey>,
    pub txn_accumulator_root: HashValue,
    pub block_accumulator_root: HashValue,
    pub state_root: HashValue,
    pub gas_used: u64,
    pub difficulty: U256,
    pub body_hash: HashValue,
    pub chain_id: ChainId,
    pub nonce: u32,
    pub extra: BlockHeaderExtra,
    pub parents_hash: Option<ParentsHash>,
    pub version: Version,
    pub pruning_point: HashValue,
}

#[derive(Serialize, Deserialize, CryptoHasher, CryptoHash)]
#[serde(rename = "BlockHeader")]
pub(crate) struct BlockHeaderDataInVega {
    pub parent_hash: HashValue,
    pub timestamp: u64,
    pub number: BlockNumber,
    pub author: AccountAddress,
    pub author_auth_key: Option<AuthenticationKey>,
    pub txn_accumulator_root: HashValue,
    pub block_accumulator_root: HashValue,
    pub state_root: HashValue,
    pub gas_used: u64,
    pub difficulty: U256,
    pub body_hash: HashValue,
    pub chain_id: ChainId,
    pub nonce: u32,
    pub extra: BlockHeaderExtra,
    pub parents_hash: Option<ParentsHash>,
}

#[derive(
    Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, CryptoHasher, CryptoHash, JsonSchema,
)]
#[serde(rename = "BlockHeader")]
struct BlockHeaderDataInVegaCrypto {
    #[serde(skip)]
    pub id: Option<HashValue>,
    pub parent_hash: HashValue,
    pub timestamp: u64,
    pub number: BlockNumber,
    pub author: AccountAddress,
    pub author_auth_key: Option<AuthenticationKey>,
    pub txn_accumulator_root: HashValue,
    pub block_accumulator_root: HashValue,
    pub state_root: HashValue,
    pub gas_used: u64,
    #[schemars(with = "String")]
    pub difficulty: U256,
    pub body_hash: HashValue,
    pub chain_id: ChainId,
    pub nonce: u32,
    pub extra: BlockHeaderExtra,
    pub parents_hash: Option<ParentsHash>,
}

impl From<BlockHeaderDataLatest> for BlockHeaderDataInVega {
    fn from(val: BlockHeaderDataLatest) -> Self {
        BlockHeaderDataInVega {
            parent_hash: val.parent_hash,
            timestamp: val.timestamp,
            number: val.number,
            author: val.author,
            author_auth_key: val.author_auth_key,
            txn_accumulator_root: val.txn_accumulator_root,
            block_accumulator_root: val.block_accumulator_root,
            state_root: val.state_root,
            gas_used: val.gas_used,
            difficulty: val.difficulty,
            body_hash: val.body_hash,
            chain_id: val.chain_id,
            nonce: val.nonce,
            extra: val.extra,
            parents_hash: val.parents_hash,
        }
    }
}

impl From<BlockHeader> for BlockHeaderDataLatest {
    fn from(val: BlockHeader) -> Self {
        BlockHeaderDataLatest {
            parent_hash: val.parent_hash,
            timestamp: val.timestamp,
            number: val.number,
            author: val.author,
            author_auth_key: val.author_auth_key,
            txn_accumulator_root: val.txn_accumulator_root,
            block_accumulator_root: val.block_accumulator_root,
            state_root: val.state_root,
            gas_used: val.gas_used,
            difficulty: val.difficulty,
            body_hash: val.body_hash,
            chain_id: val.chain_id,
            nonce: val.nonce,
            extra: val.extra,
            parents_hash: if val.parents_hash.is_empty() {
                None
            } else {
                Some(val.parents_hash)
            },
            version: val.version,
            pruning_point: val.pruning_point,
        }
    }
}

impl From<BlockHeader> for BlockHeaderDataInVega {
    fn from(val: BlockHeader) -> Self {
        BlockHeaderDataInVega {
            parent_hash: val.parent_hash,
            timestamp: val.timestamp,
            number: val.number,
            author: val.author,
            author_auth_key: val.author_auth_key,
            txn_accumulator_root: val.txn_accumulator_root,
            block_accumulator_root: val.block_accumulator_root,
            state_root: val.state_root,
            gas_used: val.gas_used,
            difficulty: val.difficulty,
            body_hash: val.body_hash,
            chain_id: val.chain_id,
            nonce: val.nonce,
            extra: val.extra,
            parents_hash: if val.parents_hash.is_empty() {
                None
            } else {
                Some(val.parents_hash)
            },
        }
    }
}

impl From<BlockHeaderBuilder> for BlockHeaderDataLatest {
    fn from(val: BlockHeaderBuilder) -> Self {
        BlockHeaderDataLatest {
            parent_hash: val.buffer.parent_hash,
            timestamp: val.buffer.timestamp,
            number: val.buffer.number,
            author: val.buffer.author,
            author_auth_key: val.buffer.author_auth_key,
            txn_accumulator_root: val.buffer.txn_accumulator_root,
            block_accumulator_root: val.buffer.block_accumulator_root,
            state_root: val.buffer.state_root,
            gas_used: val.buffer.gas_used,
            difficulty: val.buffer.difficulty,
            body_hash: val.buffer.body_hash,
            chain_id: val.buffer.chain_id,
            nonce: val.buffer.nonce,
            extra: val.buffer.extra,
            parents_hash: Some(val.buffer.parents_hash),
            version: val.buffer.version,
            pruning_point: val.buffer.pruning_point,
        }
    }
}

impl BlockHeaderDataLatest {
    pub fn into_hash(self) -> HashValue {
        if self.pruning_point == HashValue::zero() {
            // for vega hostorical blocks
            let vega_header: BlockHeaderDataInVega = self.into();
            let is_legacy = if let Some(parents_hash) = &vega_header.parents_hash {
                parents_hash.is_empty()
            } else {
                true
            };
            if is_legacy {
                let legacy_header: crate::block::legacy::BlockHeader = vega_header.into();
                legacy_header.crypto_hash()
            } else {
                let header = BlockHeaderDataInVegaCrypto {
                    id: None,
                    parent_hash: vega_header.parent_hash,
                    timestamp: vega_header.timestamp,
                    number: vega_header.number,
                    author: vega_header.author,
                    author_auth_key: vega_header.author_auth_key,
                    txn_accumulator_root: vega_header.txn_accumulator_root,
                    block_accumulator_root: vega_header.block_accumulator_root,
                    state_root: vega_header.state_root,
                    gas_used: vega_header.gas_used,
                    difficulty: vega_header.difficulty,
                    body_hash: vega_header.body_hash,
                    chain_id: vega_header.chain_id,
                    nonce: vega_header.nonce,
                    extra: vega_header.extra,
                    parents_hash: vega_header.parents_hash.and_then(|value| {
                        if value.is_empty() {
                            None
                        } else {
                            Some(value)
                        }
                    }),
                };
                header.crypto_hash()
            }
        } else {
            // for latest blocks
            self.crypto_hash()
        }
    }
}
