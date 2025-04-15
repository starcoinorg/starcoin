// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    block::{Block, BlockBody, BlockHeader},
    view::{
        block_header_view::BlockHeaderView, block_transaction_view::BlockTransactionsView,
        raw_block_view::RawBlockView,
    },
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct BlockView {
    pub header: BlockHeaderView,
    pub body: BlockTransactionsView,
    pub uncles: Vec<BlockHeaderView>,

    /// Raw block data that can be verified by block_hash and body_hash.
    pub raw: Option<RawBlockView>,
}

impl BlockView {
    pub fn try_from_block(block: Block, thin: bool, raw: bool) -> Result<Self, anyhow::Error> {
        let raw_block: Option<RawBlockView> = if raw {
            Some(RawBlockView::try_from(&block)?)
        } else {
            None
        };
        let (header, body) = block.into_inner();
        let BlockBody {
            transactions,
            uncles,
        } = body;
        let txns_view = if thin {
            BlockTransactionsView::Hashes(transactions.into_iter().map(|t| t.id()).collect())
        } else {
            transactions.try_into()?
        };
        Ok(Self {
            header: header.into(),
            uncles: uncles
                .unwrap_or_default()
                .into_iter()
                .map(|h| h.into())
                .collect(),
            body: txns_view,
            raw: raw_block,
        })
    }
}

impl TryFrom<Block> for BlockView {
    type Error = anyhow::Error;

    fn try_from(block: Block) -> Result<Self, Self::Error> {
        Self::try_from_block(block, false, false)
    }
}

impl TryFrom<BlockView> for Block {
    type Error = anyhow::Error;

    fn try_from(block_view: BlockView) -> Result<Self, Self::Error> {
        let block_header: BlockHeader = block_view.header.into();
        let uncles: Vec<BlockHeader> = block_view
            .uncles
            .into_iter()
            .map(BlockHeader::from)
            .collect();
        let transactions = block_view.body.try_into()?;

        Ok(Self {
            header: block_header,
            body: BlockBody::new(transactions, Some(uncles)),
        })
    }
}
