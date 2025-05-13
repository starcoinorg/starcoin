use serde::{Deserialize, Serialize};
use starcoin_crypto::{
    hash::{CryptoHash, CryptoHasher, PlainCryptoHash},
    HashValue,
};
use starcoin_uint::U256;
use starcoin_vm_types::{
    account_address::AccountAddress, genesis_config::ChainId,
    transaction::authenticator::AuthenticationKey,
};

use super::{BlockNumber, ParentsHash, Version};

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
    /// parents in dag
    pub parents_hash: ParentsHash,
    /// version
    pub version: Version,
    /// pruning point
    pub pruning_point: HashValue,
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, CryptoHasher, CryptoHash)]
#[serde(rename = "RawBlockHeader")]
pub struct RawBlockHeaderLegacy {
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

impl RawBlockHeader {
    pub fn calc_hash(&self) -> HashValue {
        // In vega and previous net, the parents hash is not considered to be a field which joins the pow
        if self.pruning_point == HashValue::zero() {
            let legacy_raw_header = RawBlockHeaderLegacy {
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
            };
            legacy_raw_header.crypto_hash()
        } else {
            self.crypto_hash()
        }
    }
}
