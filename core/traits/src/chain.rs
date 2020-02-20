// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::TransactionInfo;
use anyhow::Result;
use crypto::HashValue;
use types::block::{Block, BlockHeader, BlockNumber, BlockTemplate};

#[async_trait::async_trait]
pub trait Chain: Clone + std::marker::Unpin + TransactionInfo {
    async fn current_header(self) -> BlockHeader;
    async fn get_header(self, hash: &HashValue) -> BlockHeader;
    async fn get_header_by_number(self, number: BlockNumber) -> BlockHeader;
    async fn create_block_template(self) -> Result<BlockTemplate>;
    async fn get_block_by_hash(self, hash: &HashValue) -> Result<Option<Block>>;
    async fn try_connect(self, block: Block) -> Result<()>;
}
