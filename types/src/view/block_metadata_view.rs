// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{block::BlockNumber, genesis_config, view::str_view::StrView};
use move_core_types::account_address::AccountAddress;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;
use starcoin_vm_types::{
    block_metadata::BlockMetadata, transaction::authenticator::AuthenticationKey,
};

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct BlockMetadataView {
    /// Parent block hash.
    pub parent_hash: HashValue,
    pub timestamp: StrView<u64>,
    pub author: AccountAddress,
    pub author_auth_key: Option<AuthenticationKey>,
    pub uncles: StrView<u64>,
    pub number: StrView<BlockNumber>,
    pub chain_id: u8,
    pub parent_gas_used: StrView<u64>,
    pub parents_hash: Option<Vec<HashValue>>,
}

impl From<BlockMetadata> for BlockMetadataView {
    fn from(origin: BlockMetadata) -> Self {
        let (
            parent_hash,
            timestamp,
            author,
            uncles,
            number,
            chain_id,
            parent_gas_used,
            parents_hash,
        ) = origin.into_inner();
        Self {
            parent_hash,
            timestamp: timestamp.into(),
            author,
            author_auth_key: None,
            uncles: uncles.into(),
            number: number.into(),
            chain_id: chain_id.id(),
            parent_gas_used: parent_gas_used.into(),
            parents_hash,
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<BlockMetadata> for BlockMetadataView {
    fn into(self) -> BlockMetadata {
        let Self {
            parent_hash,
            timestamp,
            author,
            author_auth_key: _,
            uncles,
            number,
            chain_id,
            parent_gas_used,
            parents_hash,
        } = self;
        BlockMetadata::new_with_parents(
            parent_hash,
            timestamp.0,
            author,
            uncles.0,
            number.0,
            genesis_config::ChainId::new(chain_id),
            parent_gas_used.0,
            parents_hash.unwrap_or_default(),
        )
    }
}
