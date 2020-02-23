// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use types::block::Block;

pub trait ChainService {
    fn try_connect(&mut self, block: Block) -> Result<()>;
}

#[async_trait::async_trait]
pub trait ChainAsyncService: Clone + std::marker::Unpin {
    /// connect to head chain or a fork chain.
    async fn try_connect(self, block: Block) -> Result<()>;
}
