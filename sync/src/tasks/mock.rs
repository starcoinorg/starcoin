// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::tasks::BlockIdFetcher;
use anyhow::Result;
use futures::future::BoxFuture;
use futures::FutureExt;
use futures_timer::Delay;
use starcoin_accumulator::{Accumulator, MerkleAccumulator};
use starcoin_crypto::HashValue;
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone)]
pub struct MockBlockIdFetcher {
    accumulator: Arc<MerkleAccumulator>,
}

impl MockBlockIdFetcher {
    pub fn new(accumulator: Arc<MerkleAccumulator>) -> Self {
        Self { accumulator }
    }

    pub fn appends(&self, leaves: &[HashValue]) -> Result<()> {
        self.accumulator.append(leaves)?;
        self.accumulator.flush()
    }

    async fn fetch_block_ids_async(
        &self,
        start_number: u64,
        reverse: bool,
        max_size: usize,
    ) -> Result<Vec<HashValue>> {
        Delay::new(Duration::from_millis(100)).await;
        self.accumulator.get_leaves(start_number, reverse, max_size)
    }
}

impl BlockIdFetcher for MockBlockIdFetcher {
    fn fetch_block_ids(
        &self,
        start_number: u64,
        reverse: bool,
        max_size: usize,
    ) -> BoxFuture<Result<Vec<HashValue>>> {
        self.fetch_block_ids_async(start_number, reverse, max_size)
            .boxed()
    }
}
