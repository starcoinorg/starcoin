// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::tasks::{BlockConnectedEvent, BlockFetcher, BlockIdFetcher, FetcherFactory};
use anyhow::{format_err, Result};
use async_std::task::JoinHandle;
use futures::channel::mpsc::UnboundedReceiver;
use futures::future::BoxFuture;
use futures::{FutureExt, StreamExt};
use futures_timer::Delay;
use rand::Rng;
use starcoin_accumulator::{Accumulator, MerkleAccumulator};
use starcoin_chain_api::ChainReader;
use starcoin_chain_mock::{BlockChain, MockChain};
use starcoin_crypto::HashValue;
use starcoin_types::block::{Block, BlockInfo, BlockNumber};
use starcoin_types::peer_info::{PeerId, PeerInfo};
use starcoin_vm_types::genesis_config::ChainNetwork;
use std::sync::Arc;
use std::time::Duration;
use test_helper::DummyNetworkService;

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
        max_size: u64,
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
        max_size: u64,
    ) -> BoxFuture<Result<Vec<HashValue>>> {
        self.fetch_block_ids_async(start_number, reverse, max_size)
            .boxed()
    }

    fn fetch_block_ids_from_peer(
        &self,
        _peer: Option<PeerId>,
        start_number: BlockNumber,
        reverse: bool,
        max_size: u64,
    ) -> BoxFuture<Result<Vec<HashValue>>> {
        self.fetch_block_ids(start_number, reverse, max_size)
    }

    fn fetch_block_infos_from_peer(
        &self,
        _peer_id: Option<PeerId>,
        _hashes: Vec<HashValue>,
    ) -> BoxFuture<Result<Vec<BlockInfo>>> {
        unimplemented!()
    }

    fn find_best_peer(&self) -> Option<PeerInfo> {
        Some(PeerInfo::random())
    }
}

pub struct SyncNodeMockerFactory {
    network: DummyNetworkService,
    fetch: Arc<SyncNodeMocker>,
}

impl SyncNodeMockerFactory {
    pub fn new(network: DummyNetworkService, fetch: Arc<SyncNodeMocker>) -> Self {
        Self { network, fetch }
    }
}

impl FetcherFactory<Arc<SyncNodeMocker>, DummyNetworkService> for SyncNodeMockerFactory {
    fn create(&self, _peers: Vec<PeerInfo>) -> Arc<SyncNodeMocker> {
        self.fetch.clone()
    }

    fn network(&self) -> DummyNetworkService {
        self.network.clone()
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

    pub fn select_head(&mut self, block: Block) -> Result<()> {
        self.chain_mocker.select_head(block)
    }

    pub async fn process_block_connect_event(
        self,
        receiver: UnboundedReceiver<BlockConnectedEvent>,
    ) -> JoinHandle<Self> {
        let fut = async move {
            receiver
                .fold(self, |mut this, event| async move {
                    this.select_head(event.block).unwrap();
                    this
                })
                .await
        };
        async_std::task::spawn(fut)
    }
}

impl BlockIdFetcher for SyncNodeMocker {
    fn fetch_block_ids(
        &self,
        start_number: u64,
        reverse: bool,
        max_size: u64,
    ) -> BoxFuture<'_, Result<Vec<HashValue>>> {
        self.fetch_block_ids_from_peer(None, start_number, reverse, max_size)
    }

    fn fetch_block_ids_from_peer(
        &self,
        _peer: Option<PeerId>,
        start_number: BlockNumber,
        reverse: bool,
        max_size: u64,
    ) -> BoxFuture<Result<Vec<HashValue>>> {
        let result = self.chain().get_block_ids(start_number, reverse, max_size);
        async move {
            self.delay().await;
            self.random_err()?;
            result
        }
        .boxed()
    }

    fn fetch_block_infos_from_peer(
        &self,
        _peer_id: Option<PeerId>,
        hashes: Vec<HashValue>,
    ) -> BoxFuture<Result<Vec<BlockInfo>>> {
        let mut result: Vec<BlockInfo> = Vec::new();
        hashes.into_iter().for_each(|hash| {
            result.push(self.chain().get_block_info(Some(hash)).unwrap().unwrap());
        });
        async move {
            self.delay().await;
            self.random_err()?;
            Ok(result)
        }
        .boxed()
    }

    fn find_best_peer(&self) -> Option<PeerInfo> {
        Some(PeerInfo::random())
    }
}

impl BlockFetcher for SyncNodeMocker {
    fn fetch_block(
        &self,
        block_ids: Vec<HashValue>,
    ) -> BoxFuture<'_, Result<Vec<(Block, Option<PeerId>)>>> {
        let result: Result<Vec<(Block, Option<PeerId>)>> = block_ids
            .into_iter()
            .map(|block_id| {
                if let Some(block) = self.chain().get_block(block_id)? {
                    Ok((block, None))
                } else {
                    Err(format_err!("Can not find block by id: {}", block_id))
                }
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
