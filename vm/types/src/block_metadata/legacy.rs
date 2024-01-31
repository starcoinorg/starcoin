use crate::genesis_config::ChainId;
use crate::transaction::authenticator::AuthenticationKey;
use anyhow::anyhow;
use move_core_types::account_address::AccountAddress;
use serde::{Deserialize, Deserializer, Serialize};
use starcoin_crypto::hash::{CryptoHash, CryptoHasher, PlainCryptoHash};
use starcoin_crypto::HashValue;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, CryptoHasher, CryptoHash)]
pub struct BlockMetadata {
    #[serde(skip)]
    pub(super) id: Option<HashValue>,
    /// Parent block hash.
    pub(super) parent_hash: HashValue,
    pub(super) timestamp: u64,
    pub(super) author: AccountAddress,
    pub(super) author_auth_key: Option<AuthenticationKey>,
    pub(super) uncles: u64,
    pub(super) number: u64,
    pub(super) chain_id: ChainId,
    pub(super) parent_gas_used: u64,
}

impl<'de> Deserialize<'de> for BlockMetadata {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(rename = "BlockMetadata")]
        struct BlockMetadataData {
            parent_hash: HashValue,
            timestamp: u64,
            author: AccountAddress,
            author_auth_key: Option<AuthenticationKey>,
            uncles: u64,
            number: u64,
            chain_id: ChainId,
            parent_gas_used: u64,
        }
        let data = BlockMetadataData::deserialize(deserializer)?;
        let mut txn = Self {
            id: None,
            parent_hash: data.parent_hash,
            timestamp: data.timestamp,
            author: data.author,
            author_auth_key: data.author_auth_key,
            uncles: data.uncles,
            number: data.number,
            chain_id: data.chain_id,
            parent_gas_used: data.parent_gas_used,
        };
        txn.id = Some(txn.crypto_hash());
        Ok(txn)
    }
}

impl From<BlockMetadata> for super::BlockMetadata {
    fn from(value: BlockMetadata) -> Self {
        Self {
            id: value.id,
            parent_hash: value.parent_hash,
            timestamp: value.timestamp,
            author: value.author,
            author_auth_key: value.author_auth_key,
            uncles: value.uncles,
            number: value.number,
            chain_id: value.chain_id,
            parent_gas_used: value.parent_gas_used,
            parents_hash: None,
        }
    }
}

impl TryFrom<super::BlockMetadata> for BlockMetadata {
    type Error = anyhow::Error;

    fn try_from(value: super::BlockMetadata) -> Result<Self, Self::Error> {
        if value.parents_hash.is_some() {
            return Err(anyhow!(
                "Can't convert a new BlockMetaData txn with parents_hash to an old one"
            ));
        }
        Ok(Self {
            id: value.id,
            parent_hash: value.parent_hash,
            timestamp: value.timestamp,
            author: value.author,
            author_auth_key: value.author_auth_key,
            uncles: value.uncles,
            number: value.number,
            chain_id: value.chain_id,
            parent_gas_used: value.parent_gas_used,
        })
    }
}
