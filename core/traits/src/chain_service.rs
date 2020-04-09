// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use crypto::HashValue;
use types::{
    account_address::AccountAddress,
    block::{Block, BlockHeader, BlockInfo, BlockNumber, BlockTemplate},
    startup_info::StartupInfo,
    transaction::SignedUserTransaction,
    U256,
};

/// implement ChainService
pub trait ChainService {
    /////////////////////////////////////////////// for chain service
    fn try_connect(&mut self, block: Block) -> Result<()>;
    fn get_header_by_hash(&self, hash: HashValue) -> Result<Option<BlockHeader>>;
    fn get_block_by_hash(&self, hash: HashValue) -> Result<Option<Block>>;
    fn try_connect_with_block_info(&mut self, block: Block, block_info: BlockInfo) -> Result<()>;
    fn get_block_info_by_hash(&self, hash: HashValue) -> Result<Option<BlockInfo>>;

    /////////////////////////////////////////////// for master
    fn master_head_header(&self) -> BlockHeader;
    fn master_head_block(&self) -> Block;
    fn master_block_by_number(&self, number: BlockNumber) -> Result<Option<Block>>;
    fn master_startup_info(&self) -> StartupInfo;

    /////////////////////////////////////////////// just for test
    fn create_block_template(
        &self,
        author: AccountAddress,
        auth_key_prefix: Option<Vec<u8>>,
        parent_hash: Option<HashValue>,
        difficulty: U256,
        user_txns: Vec<SignedUserTransaction>,
    ) -> Result<BlockTemplate>;
    fn gen_tx(&self) -> Result<()>;
}

/// ChainActor
#[async_trait::async_trait(? Send)]
pub trait ChainAsyncService: Clone + std::marker::Unpin {
    /////////////////////////////////////////////// for chain service
    /// connect to master or a fork branch.
    async fn try_connect(self, block: Block) -> Result<()>;
    async fn get_header_by_hash(self, hash: &HashValue) -> Option<BlockHeader>;
    async fn get_block_by_hash(self, hash: &HashValue) -> Option<Block>;
    async fn try_connect_with_block_info(
        &mut self,
        block: Block,
        block_info: BlockInfo,
    ) -> Result<()>;
    async fn get_block_info_by_hash(self, hash: &HashValue) -> Option<BlockInfo>;

    /////////////////////////////////////////////// for master
    async fn master_head_header(self) -> Option<BlockHeader>;
    async fn master_head_block(self) -> Option<Block>;
    async fn master_block_by_number(self, number: BlockNumber) -> Option<Block>;
    async fn master_startup_info(self) -> Result<StartupInfo>;

    /////////////////////////////////////////////// just for test
    async fn gen_tx(&self) -> Result<()>;
    async fn create_block_template(
        self,
        author: AccountAddress,
        auth_key_prefix: Option<Vec<u8>>,
        parent_hash: Option<HashValue>,
        txs: Vec<SignedUserTransaction>,
        difficulty: U256,
    ) -> Option<BlockTemplate>;
}
