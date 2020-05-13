// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::ConnectResult;
use anyhow::Result;
use starcoin_crypto::HashValue;
use starcoin_types::startup_info::ChainInfo;
use starcoin_types::transaction::TransactionInfo;
use starcoin_types::{
    account_address::AccountAddress,
    block::{Block, BlockHeader, BlockInfo, BlockNumber, BlockTemplate},
    startup_info::StartupInfo,
    transaction::SignedUserTransaction,
};

/// implement ChainService
pub trait ChainService {
    /////////////////////////////////////////////// for chain service
    fn try_connect(&mut self, block: Block, pivot_sync: bool) -> Result<ConnectResult<()>>;
    fn get_header_by_hash(&self, hash: HashValue) -> Result<Option<BlockHeader>>;
    fn get_block_by_hash(&self, hash: HashValue) -> Result<Option<Block>>;
    fn try_connect_with_block_info(
        &mut self,
        block: Block,
        block_info: BlockInfo,
    ) -> Result<ConnectResult<()>>;
    fn get_block_info_by_hash(&self, hash: HashValue) -> Result<Option<BlockInfo>>;

    /////////////////////////////////////////////// for master
    fn master_head_header(&self) -> BlockHeader;
    fn master_head_block(&self) -> Block;
    fn master_block_by_number(&self, number: BlockNumber) -> Result<Option<Block>>;
    fn master_startup_info(&self) -> StartupInfo;
    fn master_blocks_by_number(
        &self,
        number: Option<BlockNumber>,
        count: u64,
    ) -> Result<Vec<Block>>;
    fn get_transaction(&self, hash: HashValue) -> Result<Option<TransactionInfo>>;
    fn get_block_txn_ids(&self, block_id: HashValue) -> Result<Vec<TransactionInfo>>;
    /////////////////////////////////////////////// just for test
    fn create_block_template(
        &self,
        author: AccountAddress,
        auth_key_prefix: Option<Vec<u8>>,
        parent_hash: Option<HashValue>,
        user_txns: Vec<SignedUserTransaction>,
    ) -> Result<BlockTemplate>;
}

/// ChainActor
#[async_trait::async_trait]
pub trait ChainAsyncService:
    Clone + std::marker::Unpin + std::marker::Sync + std::marker::Send
{
    /////////////////////////////////////////////// for chain service
    /// connect to master or a fork branch.
    async fn try_connect(self, block: Block) -> Result<ConnectResult<()>>;
    async fn get_header_by_hash(self, hash: &HashValue) -> Option<BlockHeader>;
    async fn get_block_by_hash(self, hash: HashValue) -> Result<Block>;
    async fn try_connect_with_block_info(
        &mut self,
        block: Block,
        block_info: BlockInfo,
    ) -> Result<ConnectResult<()>>;
    async fn get_block_info_by_hash(self, hash: &HashValue) -> Option<BlockInfo>;

    /////////////////////////////////////////////// for master
    async fn master_head_header(self) -> Option<BlockHeader>;
    async fn master_head_block(self) -> Option<Block>;
    async fn master_block_by_number(self, number: BlockNumber) -> Result<Block>;
    async fn master_blocks_by_number(
        self,
        number: Option<BlockNumber>,
        count: u64,
    ) -> Result<Vec<Block>>;
    async fn master_startup_info(self) -> Result<StartupInfo>;
    async fn master_head(self) -> Result<ChainInfo>;
    async fn get_transaction(self, txn_id: HashValue) -> Result<TransactionInfo>;
    async fn get_block_txn(self, block_id: HashValue) -> Result<Vec<TransactionInfo>>;
    /////////////////////////////////////////////// just for test
    async fn create_block_template(
        self,
        author: AccountAddress,
        auth_key_prefix: Option<Vec<u8>>,
        parent_hash: Option<HashValue>,
        txs: Vec<SignedUserTransaction>,
    ) -> Option<BlockTemplate>;
}
