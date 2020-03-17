// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use crypto::HashValue;
use types::{
    block::{Block, BlockHeader, BlockNumber, BlockTemplate},
    startup_info::ChainInfo,
    transaction::SignedUserTransaction,
};

pub trait ChainService {
    fn try_connect(&mut self, block: Block) -> Result<()>;
    fn get_head_branch(&self) -> HashValue;
}

#[async_trait::async_trait(? Send)]
pub trait ChainAsyncService: Clone + std::marker::Unpin {
    /// connect to head chain or a fork chain.
    async fn try_connect(self, block: Block) -> Result<()>;
    async fn get_head_branch(self) -> Result<HashValue>;
    async fn get_chain_info(self) -> Result<ChainInfo>;
    async fn gen_tx(&self) -> Result<()>;

    ////////////////////////////////////////////
    async fn current_header(self) -> Option<BlockHeader>;
    async fn get_header_by_hash(self, hash: &HashValue) -> Option<BlockHeader>;
    async fn head_block(self) -> Option<Block>;
    async fn get_header_by_number(self, number: BlockNumber) -> Option<BlockHeader>;
    async fn get_block_by_number(self, number: BlockNumber) -> Option<Block>;
    async fn create_block_template(
        self,
        parent_hash: Option<HashValue>,
        txs: Vec<SignedUserTransaction>,
    ) -> Option<BlockTemplate>;
    async fn get_block_by_hash(self, hash: &HashValue) -> Option<Block>;
}
