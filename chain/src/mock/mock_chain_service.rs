// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{ChainAsyncService, ConnectResult};
use anyhow::{Error, Result};
use crypto::HashValue;
use types::{
    account_address::AccountAddress,
    block::{Block, BlockHeader, BlockInfo, BlockNumber, BlockState, BlockTemplate},
    startup_info::{ChainInfo, StartupInfo},
    transaction::{SignedUserTransaction, TransactionInfo},
};

//TODO implement Mock service
#[derive(Clone, Default)]
pub struct MockChainService;

#[async_trait::async_trait]
impl ChainAsyncService for MockChainService {
    async fn try_connect(self, _block: Block) -> Result<ConnectResult<()>> {
        unimplemented!()
    }

    async fn get_header_by_hash(self, _hash: &HashValue) -> Result<Option<BlockHeader>> {
        unimplemented!()
    }

    async fn get_block_by_hash(self, _hash: HashValue) -> Result<Block> {
        unimplemented!()
    }

    async fn get_block_state_by_hash(self, _hash: &HashValue) -> Result<Option<BlockState>> {
        unimplemented!()
    }

    async fn master_block_header_by_number(self, _number: BlockNumber) -> Result<BlockHeader> {
        unimplemented!()
    }

    async fn try_connect_with_block_info(
        &mut self,
        _block: Block,
        _block_info: BlockInfo,
    ) -> Result<ConnectResult<()>, Error> {
        unimplemented!()
    }

    async fn get_block_info_by_hash(self, _hash: &HashValue) -> Result<Option<BlockInfo>> {
        unimplemented!()
    }

    async fn master_head_header(self) -> Result<Option<BlockHeader>> {
        unimplemented!()
    }

    async fn master_head_block(self) -> Result<Option<Block>> {
        unimplemented!()
    }

    async fn master_block_by_number(self, _number: u64) -> Result<Block> {
        unimplemented!()
    }

    async fn master_blocks_by_number(
        self,
        _number: Option<BlockNumber>,
        _count: u64,
    ) -> Result<Vec<Block>> {
        unimplemented!()
    }

    async fn master_startup_info(self) -> Result<StartupInfo> {
        unimplemented!()
    }

    async fn master_head(self) -> Result<ChainInfo> {
        unimplemented!()
    }

    async fn get_transaction(self, _txn_id: HashValue) -> Result<TransactionInfo> {
        unimplemented!()
    }

    async fn get_block_txn(self, _block_id: HashValue) -> Result<Vec<TransactionInfo>> {
        unimplemented!()
    }

    async fn create_block_template(
        self,
        _author: AccountAddress,
        _auth_key_prefix: Option<Vec<u8>>,
        _parent_hash: Option<HashValue>,
        _txs: Vec<SignedUserTransaction>,
    ) -> Result<Option<BlockTemplate>> {
        unimplemented!()
    }
}
