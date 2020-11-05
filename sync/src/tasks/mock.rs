// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::tasks::{BlockFetcher, BlockIdFetcher};
use anyhow::{format_err, Result};
use futures::future::BoxFuture;
use futures::FutureExt;
use futures_timer::Delay;
use logger::prelude::*;
use rand::Rng;
use starcoin_accumulator::{Accumulator, MerkleAccumulator};
use starcoin_chain_api::ChainReader;
use starcoin_chain_mock::{BlockChain, MockChain};
use starcoin_crypto::HashValue;
use starcoin_types::block::Block;
use starcoin_vm_types::genesis_config::ChainNetwork;
use starcoin_vm_types::on_chain_resource::GlobalTimeOnChain;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use stream_task::{CollectorState, TaskResultCollector};

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

pub struct SyncNodeMocker {
    pub chain_mocker: MockChain,
    pub delay_milliseconds: u64,
    pub random_error_percent: u32,
}

impl SyncNodeMocker {
    pub fn new(
        net: ChainNetwork,
        delay_milliseconds: u64,
        random_error_percent: u32,
    ) -> Result<Self> {
        Ok(Self {
            chain_mocker: MockChain::new(net)?,
            delay_milliseconds,
            random_error_percent,
        })
    }

    pub fn chain(&self) -> &BlockChain {
        self.chain_mocker.head()
    }

    pub fn produce_block(&mut self, times: u64) -> Result<()> {
        self.chain_mocker.produce_and_apply_times(times)
    }

    async fn delay(&self) {
        if self.delay_milliseconds > 0 {
            Delay::new(Duration::from_millis(self.delay_milliseconds)).await
        }
    }
    fn random_err(&self) -> Result<()> {
        if self.random_error_percent > 0 {
            let rnd = rand::thread_rng().gen_range(0, 100);
            if rnd <= self.random_error_percent {
                return Err(format_err!("Random error {}", rnd));
            }
        }
        Ok(())
    }
}

impl BlockIdFetcher for SyncNodeMocker {
    fn fetch_block_ids(
        &self,
        start_number: u64,
        reverse: bool,
        max_size: usize,
    ) -> BoxFuture<'_, Result<Vec<HashValue>>> {
        let result = self.chain().get_block_ids(start_number, reverse, max_size);
        async move {
            self.delay().await;
            self.random_err()?;
            result
        }
        .boxed()
    }
}

impl BlockFetcher for SyncNodeMocker {
    fn fetch_block(&self, block_ids: Vec<HashValue>) -> BoxFuture<'_, Result<Vec<Block>>> {
        let result: Result<Vec<Block>> = block_ids
            .into_iter()
            .map(|block_id| {
                self.chain()
                    .get_block(block_id)?
                    .ok_or_else(|| format_err!("Can not find block by id: {}", block_id))
            })
            .collect();
        async move {
            self.delay().await;
            self.random_err()?;
            result
        }
        .boxed()
    }
}

impl TaskResultCollector<Block> for SyncNodeMocker {
    type Output = Self;

    fn collect(mut self: Pin<&mut Self>, item: Block) -> Result<CollectorState> {
        debug!("Collect block: {}", item);
        let timestamp = item.header().timestamp();
        if self.chain().current_header().id() != item.header().parent_hash() {
            self.chain_mocker = self.chain_mocker.fork(Some(item.header().parent_hash()))?;
        }
        self.chain_mocker.apply(item)?;
        self.chain_mocker
            .net()
            .time_service()
            .adjust(GlobalTimeOnChain::new(timestamp));
        Ok(CollectorState::Need)
    }

    fn finish(self) -> Result<Self::Output> {
        Ok(self)
    }
}
