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
    pub parents_hash: ParentsHash,
    pub version: Version,
    pub pruning_point: HashValue,
}

impl From<BlockHeader> for BlockHeaderDataLatest {
    fn from(val: BlockHeader) -> Self {
        Self {
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
            version: val.version,
            pruning_point: val.pruning_point,
        }
    }
}

impl From<BlockHeaderBuilder> for BlockHeaderDataLatest {
    fn from(val: BlockHeaderBuilder) -> Self {
        Self {
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
            parents_hash: val.buffer.parents_hash,
            version: val.buffer.version,
            pruning_point: val.buffer.pruning_point,
        }
    }
}

impl BlockHeaderDataLatest {
    pub fn into_hash(self) -> HashValue {
        // for latest blocks
        self.crypto_hash()
    }
}
