// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use crypto::HashValue;
use types::{block::Block, startup_info::ChainInfo};

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
}
