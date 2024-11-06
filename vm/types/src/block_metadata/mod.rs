// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod legacy;

use crate::account_address::AccountAddress;
use crate::account_config::genesis_address;
use crate::genesis_config::ChainId;
use bcs_ext::Sample;
pub use legacy::BlockMetadata as LegacyBlockMetadata;
use serde::{Deserialize, Deserializer, Serialize};
use starcoin_crypto::hash::PlainCryptoHash;
use starcoin_crypto::{
    hash::{CryptoHash, CryptoHasher},
    HashValue,
};

/// Struct that will be persisted on chain to store the information of the current block.
///
/// The flow will look like following:
/// 1. The executor will pass this struct to VM at the begin of a block proposal.
/// 2. The VM will use this struct to create a special system transaction that will modify the on
///    chain resource that represents the information of the current block. This transaction can't
///    be emitted by regular users and is generated by each of the miners on the fly. Such
///    transaction will be executed before all of the user-submitted transactions in the blocks.
/// 3. Once that special resource is modified, the other user transactions can read the consensus
///    info by calling into the read method of that resource, which would thus give users the
///    information such as the current block number.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, CryptoHasher, CryptoHash)]
//TODO rename to BlockMetadataTransaction
pub struct BlockMetadata {
    #[serde(skip)]
    id: Option<HashValue>,
    /// Parent block hash.
    parent_hash: HashValue,
    timestamp: u64,
    author: AccountAddress,
    uncles: u64,
    number: u64,
    chain_id: ChainId,
    parent_gas_used: u64,
    parents_hash: Option<Vec<HashValue>>,
}

impl BlockMetadata {
    pub fn new(
        parent_hash: HashValue,
        timestamp: u64,
        author: AccountAddress,
        uncles: u64,
        number: u64,
        chain_id: ChainId,
        parent_gas_used: u64,
    ) -> Self {
        let mut txn = legacy::BlockMetadata {
            id: None,
            parent_hash,
            timestamp,
            author,
            uncles,
            number,
            chain_id,
            parent_gas_used,
        };
        txn.id = Some(txn.crypto_hash());
        txn.into()
    }

    pub fn new_with_parents(
        parent_hash: HashValue,
        timestamp: u64,
        author: AccountAddress,
        uncles: u64,
        number: u64,
        chain_id: ChainId,
        parent_gas_used: u64,
        parents_hash: Vec<HashValue>,
    ) -> Self {
        let mut txn = Self {
            id: None,
            parent_hash,
            timestamp,
            author,
            uncles,
            number,
            chain_id,
            parent_gas_used,
            parents_hash: Some(parents_hash),
        };
        txn.id = Some(txn.crypto_hash());
        txn
    }

    pub fn into_inner(
        self,
    ) -> (
        HashValue,
        u64,
        AccountAddress,
        u64,
        u64,
        ChainId,
        u64,
        Option<Vec<HashValue>>,
    ) {
        (
            self.parent_hash,
            self.timestamp,
            self.author,
            self.uncles,
            self.number,
            self.chain_id,
            self.parent_gas_used,
            self.parents_hash,
        )
    }

    pub fn parent_hash(&self) -> HashValue {
        self.parent_hash
    }

    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    pub fn number(&self) -> u64 {
        self.number
    }

    pub fn chain_id(&self) -> ChainId {
        self.chain_id
    }

    pub fn id(&self) -> HashValue {
        self.id
            .expect("BlockMetadata's id should been Some after init.")
    }

    pub fn author(&self) -> AccountAddress {
        self.author
    }
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
            uncles: u64,
            number: u64,
            chain_id: ChainId,
            parent_gas_used: u64,
            parents_hash: Option<Vec<HashValue>>,
        }
        let data = BlockMetadataData::deserialize(deserializer)?;
        Ok(if let Some(parents_hash) = data.parents_hash {
            Self::new_with_parents(
                data.parent_hash,
                data.timestamp,
                data.author,
                data.uncles,
                data.number,
                data.chain_id,
                data.parent_gas_used,
                parents_hash,
            )
        } else {
            Self::new(
                data.parent_hash,
                data.timestamp,
                data.author,
                data.uncles,
                data.number,
                data.chain_id,
                data.parent_gas_used,
            )
        })
    }
}

impl Sample for BlockMetadata {
    fn sample() -> Self {
        Self::new_with_parents(
            HashValue::zero(),
            0,
            genesis_address(),
            0,
            0,
            ChainId::test(),
            0,
            vec![],
        )
    }
}
