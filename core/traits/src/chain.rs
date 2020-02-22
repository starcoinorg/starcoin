// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use crypto::HashValue;
use types::block::{Block, BlockHeader, BlockNumber, BlockTemplate};

#[async_trait::async_trait]
pub trait Chain: Clone + std::marker::Unpin {
    async fn current_header(self) -> Option<BlockHeader>;
    async fn get_header_by_hash(self, hash: &HashValue) -> Option<BlockHeader>;
    async fn head_block(self) -> Option<Block>;
    async fn get_header_by_number(self, number: BlockNumber) -> Option<BlockHeader>;
    async fn get_block_by_number(self, number: BlockNumber) -> Option<Block>;
    async fn create_block_template(self) -> Option<BlockTemplate>;
    async fn get_block_by_hash(self, hash: &HashValue) -> Option<Block>;
    async fn try_connect(self, block: Block) -> Result<()>;
}
