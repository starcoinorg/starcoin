// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    block::{BlockHeader, BlockHeaderExtra, BlockNumber, ParentsHash},
    genesis_config,
    view::str_view::StrView,
};
use move_core_types::account_address::AccountAddress;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;
use starcoin_uint::U256;

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct BlockHeaderView {
    pub block_hash: HashValue,
    /// Parent hash.
    pub parent_hash: HashValue,
    /// Block timestamp.
    pub timestamp: StrView<u64>,
    /// Block number.
    pub number: StrView<BlockNumber>,
    /// Block author.
    pub author: AccountAddress,
    /// The transaction accumulator root hash after executing this block.
    pub txn_accumulator_root: HashValue,
    /// The block accumulator root hash.
    pub block_accumulator_root: HashValue,
    /// The last transaction state_root of this block after execute.
    pub state_root: HashValue,
    /// Gas used for contracts execution.
    pub gas_used: StrView<u64>,
    /// Block difficulty
    #[schemars(with = "String")]
    pub difficulty: U256,
    /// hash for block body
    pub body_hash: HashValue,
    /// The chain id
    pub chain_id: u8,
    /// Consensus nonce field.
    pub nonce: u32,
    /// block header extra
    pub extra: BlockHeaderExtra,
    /// block parents
    pub parents_hash: ParentsHash,
}

impl From<BlockHeader> for BlockHeaderView {
    fn from(origin: BlockHeader) -> Self {
        Self {
            block_hash: origin.id(),
            parent_hash: origin.parent_hash(),
            timestamp: origin.timestamp().into(),
            number: origin.number().into(),
            author: origin.author(),
            txn_accumulator_root: origin.txn_accumulator_root(),
            block_accumulator_root: origin.block_accumulator_root(),
            state_root: origin.state_root(),
            gas_used: origin.gas_used().into(),
            difficulty: origin.difficulty(),
            body_hash: origin.body_hash(),
            chain_id: origin.chain_id().id(),
            nonce: origin.nonce(),
            extra: *origin.extra(),
            parents_hash: origin.parents_hash(),
        }
    }
}

impl From<BlockHeaderView> for BlockHeader {
    fn from(header_view: BlockHeaderView) -> Self {
        Self::new(
            header_view.parent_hash,
            header_view.timestamp.0,
            header_view.number.0,
            header_view.author,
            header_view.txn_accumulator_root,
            header_view.block_accumulator_root,
            header_view.state_root,
            header_view.gas_used.0,
            header_view.difficulty,
            header_view.body_hash,
            genesis_config::ChainId::new(header_view.chain_id),
            header_view.nonce,
            header_view.extra,
            header_view.parents_hash,
        )
    }
}

impl FromIterator<BlockHeaderView> for Vec<BlockHeader> {
    fn from_iter<T: IntoIterator<Item = BlockHeaderView>>(views: T) -> Self {
        let mut blocks = vec![];
        for view in views {
            blocks.push(view.into())
        }
        blocks
    }
}
