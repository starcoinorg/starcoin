// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::ChainAsyncService;
use crate::ConnectResult;
use anyhow::{Error, Result};
use crypto::HashValue;
use types::startup_info::ChainInfo;
use types::{
    account_address::AccountAddress,
    block::{Block, BlockHeader, BlockInfo, BlockTemplate},
    startup_info::StartupInfo,
    transaction::SignedUserTransaction,
};

//TODO implement Mock service
#[derive(Clone)]
pub struct MockChainService {}

impl MockChainService {
    pub fn new() -> MockChainService {
        Self {}
    }
}

#[async_trait::async_trait]
impl ChainAsyncService for MockChainService {
    async fn try_connect(self, _block: Block) -> Result<ConnectResult<()>, Error> {
        unimplemented!()
    }

    async fn get_header_by_hash(self, _hash: &HashValue) -> Option<BlockHeader> {
        unimplemented!()
    }

    async fn get_block_by_hash(self, _hash: HashValue) -> Result<Block, Error> {
        unimplemented!()
    }

    async fn try_connect_with_block_info(
        &mut self,
        _block: Block,
        _block_info: BlockInfo,
    ) -> Result<ConnectResult<()>, Error> {
        unimplemented!()
    }

    async fn get_block_info_by_hash(self, _hash: &HashValue) -> Option<BlockInfo> {
        unimplemented!()
    }

    async fn master_head_header(self) -> Option<BlockHeader> {
        unimplemented!()
    }

    async fn master_head_block(self) -> Option<Block> {
        unimplemented!()
    }

    async fn master_block_by_number(self, _number: u64) -> Option<Block> {
        unimplemented!()
    }

    async fn master_startup_info(self) -> Result<StartupInfo, Error> {
        unimplemented!()
    }

    async fn master_head(self) -> Result<ChainInfo, Error> {
        unimplemented!()
    }

    async fn gen_tx(&self) -> Result<(), Error> {
        unimplemented!()
    }

    async fn create_block_template(
        self,
        _author: AccountAddress,
        _auth_key_prefix: Option<Vec<u8>>,
        _parent_hash: Option<HashValue>,
        _txs: Vec<SignedUserTransaction>,
    ) -> Option<BlockTemplate> {
        unimplemented!()
    }
}
