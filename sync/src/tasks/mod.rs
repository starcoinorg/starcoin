// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::verified_rpc_client::VerifiedRpcClient;
use anyhow::Result;
use futures::future::BoxFuture;
use futures::FutureExt;
use starcoin_crypto::HashValue;
use starcoin_types::block::{Block, BlockNumber};

pub trait BlockIdFetcher: Send + Sync {
    fn fetch_block_ids(
        &self,
        start_number: BlockNumber,
        reverse: bool,
        max_size: usize,
    ) -> BoxFuture<Result<Vec<HashValue>>>;
}

impl BlockIdFetcher for VerifiedRpcClient {
    fn fetch_block_ids(
        &self,
        start_number: u64,
        reverse: bool,
        max_size: usize,
    ) -> BoxFuture<Result<Vec<HashValue>>> {
        self.get_block_ids(start_number, reverse, max_size).boxed()
    }
}

pub trait BlockFetcher: Send + Sync {
    fn fetch(&self, block_ids: Vec<HashValue>) -> BoxFuture<Result<Vec<Block>>>;
}

mod accumulator_sync_task;
mod block_sync_task;
mod find_ancestor_task;
#[cfg(test)]
pub(crate) mod mock;

pub use accumulator_sync_task::{AccumulatorCollector, BlockAccumulatorSyncTask};
pub use block_sync_task::BlockSyncTask;
pub use find_ancestor_task::{AncestorCollector, FindAncestorTask};
