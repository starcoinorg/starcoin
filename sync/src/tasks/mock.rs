// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::tasks::{
    BlockConnectedEvent, BlockFetcher, BlockIdFetcher, BlockInfoFetcher, PeerOperator, SyncFetcher,
};
use anyhow::{format_err, Context, Result};
use async_std::task::JoinHandle;
use config::ChainNetwork;
use futures::channel::mpsc::UnboundedReceiver;
use futures::future::BoxFuture;
use futures::{FutureExt, StreamExt};
use futures_timer::Delay;
use network_api::messages::NotificationMessage;
use network_api::{PeerInfo, PeerSelector, PeerStrategy};
use network_rpc_core::{NetRpcError, RpcErrorCode};
use rand::Rng;
use starcoin_accumulator::{Accumulator, MerkleAccumulator};
use starcoin_chain::BlockChain;
use starcoin_chain_api::ChainReader;
use starcoin_chain_mock::MockChain;
use starcoin_crypto::HashValue;
use starcoin_network_rpc_api::RPC_INFO;
use starcoin_sync_api::SyncTarget;
use starcoin_types::block::{Block, BlockIdAndNumber, BlockInfo, BlockNumber};
use starcoin_types::peer_info::PeerId;
use std::sync::Arc;
use std::time::Duration;

pub enum ErrorStrategy {
    _RateLimitErr,
    Timeout(u64),
    RandomErr,
    MethodNotFound,
}

impl Default for ErrorStrategy {
    fn default() -> Self {
        ErrorStrategy::RandomErr
    }
}

pub struct ErrorMocker {
    strategy: ErrorStrategy,
    pub random_error_percent: u32,
    pub peer_id: PeerId,
}

impl ErrorMocker {
    pub fn new(strategy: ErrorStrategy, error_percent: u32, peer_id: PeerId) -> Self {
        Self {
            strategy,
            random_error_percent: error_percent,
            peer_id,
        }
    }

    async fn delay(delay_milliseconds: u64) {
        if delay_milliseconds > 0 {
            Delay::new(Duration::from_millis(delay_milliseconds)).await
        }
    }

    pub async fn random_err(&self) -> Result<()> {
        if self.random_error_percent > 0 {
            let rnd = rand::thread_rng().gen_range(0..100);
            if rnd <= self.random_error_percent {
                return match &self.strategy {
                    ErrorStrategy::RandomErr => Err(format_err!("Random error {}", rnd)),
                    ErrorStrategy::Timeout(delay_milliseconds) => {
                        Self::delay(*delay_milliseconds).await;
                        Err(format_err!("Timeout error {}", rnd))
                    }
                    ErrorStrategy::_RateLimitErr => Err(NetRpcError::new(
                        RpcErrorCode::RateLimited,
                        "RateLimit".to_string(),
                    )
                    .into()),
                    ErrorStrategy::MethodNotFound => {
                        let rpc_error = NetRpcError::new(
                            RpcErrorCode::MethodNotFound,
                            "MethodNotFound".to_string(),
                        );
                        let error = Err(rpc_error);
                        error.with_context(|| self.peer_id.clone())
                    }
                };
            }
        }

        Ok(())
    }
}

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
        _peer: Option<PeerId>,
        start_number: BlockNumber,
        reverse: bool,
        max_size: u64,
    ) -> BoxFuture<Result<Vec<HashValue>>> {
        self.fetch_block_ids_async(start_number, reverse, max_size)
            .boxed()
    }
}

pub struct SyncNodeMocker {
    pub peer_id: PeerId,
    pub chain_mocker: MockChain,
    pub err_mocker: ErrorMocker,
    peer_selector: PeerSelector,
}

impl SyncNodeMocker {
    pub fn new(
        net: ChainNetwork,
        delay_milliseconds: u64,
        random_error_percent: u32,
    ) -> Result<Self> {
        let chain = MockChain::new(net)?;
        let peer_id = PeerId::random();
        let peer_info = PeerInfo::new(
            peer_id.clone(),
            chain.chain_info(),
            NotificationMessage::protocols(),
            RPC_INFO.clone().into_protocols(),
        );
        let peer_selector = PeerSelector::new(vec![peer_info], PeerStrategy::default(), None);
        Ok(Self::new_inner(
            peer_id,
            chain,
            ErrorStrategy::Timeout(delay_milliseconds),
            random_error_percent,
            peer_selector,
        ))
    }

    pub fn new_with_strategy(
        net: ChainNetwork,
        error_strategy: ErrorStrategy,
        random_error_percent: u32,
    ) -> Result<Self> {
        let chain = MockChain::new(net)?;
        let peer_id = PeerId::random();
        let peer_info = PeerInfo::new(peer_id.clone(), chain.chain_info(), vec![], vec![]);
        let peer_selector = PeerSelector::new(vec![peer_info], PeerStrategy::default(), None);
        Ok(Self::new_inner(
            peer_id,
            chain,
            error_strategy,
            random_error_percent,
            peer_selector,
        ))
    }

    pub fn new_with_chain_selector(
        peer_id: PeerId,
        chain: MockChain,
        delay_milliseconds: u64,
        random_error_percent: u32,
        peer_selector: PeerSelector,
    ) -> Self {
        Self::new_inner(
            peer_id,
            chain,
            ErrorStrategy::Timeout(delay_milliseconds),
            random_error_percent,
            peer_selector,
        )
    }

    fn new_inner(
        peer_id: PeerId,
        chain: MockChain,
        error_strategy: ErrorStrategy,
        random_error_percent: u32,
        peer_selector: PeerSelector,
    ) -> Self {
        Self {
            peer_id: peer_id.clone(),
            chain_mocker: chain,
            err_mocker: ErrorMocker::new(error_strategy, random_error_percent, peer_id),
            peer_selector,
        }
    }

    pub fn peer_info(&self) -> PeerInfo {
        PeerInfo::new(
            self.peer_id.clone(),
            self.chain_mocker.chain_info(),
            vec![],
            vec![],
        )
    }

    pub fn sync_target(&self) -> SyncTarget {
        let status = self.chain().status();
        SyncTarget {
            target_id: BlockIdAndNumber::new(status.head.id(), status.head.number()),
            block_info: status.info,
            peers: vec![self.peer_id.clone()],
        }
    }

    pub fn sync_target_by_number(&self, block_number: BlockNumber) -> Result<SyncTarget> {
        let block = self
            .chain()
            .get_block_by_number(block_number)?
            .ok_or_else(|| format_err!("Can not find block by number: {}", block_number))?;
        let block_info = self
            .chain()
            .get_block_info(Some(block.id()))?
            .ok_or_else(|| format_err!("Can not find block info by id: {}", block.id()))?;
        Ok(SyncTarget {
            target_id: BlockIdAndNumber::new(block.header().id(), block.header().number()),
            block_info,
            peers: vec![self.peer_id.clone()],
        })
    }

    pub fn chain(&self) -> &BlockChain {
        self.chain_mocker.head()
    }

    pub fn produce_block(&mut self, times: u64) -> Result<()> {
        self.chain_mocker.produce_and_apply_times(times)
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

    pub fn select_a_peer(&self) -> Result<PeerId> {
        self.peer_selector
            .select_peer()
            .ok_or_else(|| format_err!("No peers for send request."))
    }
}

impl PeerOperator for SyncNodeMocker {
    fn peer_selector(&self) -> PeerSelector {
        self.peer_selector.clone()
    }
}

impl BlockIdFetcher for SyncNodeMocker {
    fn fetch_block_ids(
        &self,
        _peer: Option<PeerId>,
        start_number: BlockNumber,
        reverse: bool,
        max_size: u64,
    ) -> BoxFuture<Result<Vec<HashValue>>> {
        let result = self.chain().get_block_ids(start_number, reverse, max_size);
        async move {
            let _ = self.select_a_peer()?;
            self.err_mocker.random_err().await?;
            result
        }
        .boxed()
    }
}

impl BlockFetcher for SyncNodeMocker {
    fn fetch_blocks(
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
            let _ = self.select_a_peer()?;
            self.err_mocker.random_err().await?;
            result
        }
        .boxed()
    }
}

impl BlockInfoFetcher for SyncNodeMocker {
    fn fetch_block_infos(
        &self,
        _peer_id: Option<PeerId>,
        block_ids: Vec<HashValue>,
    ) -> BoxFuture<Result<Vec<Option<BlockInfo>>>> {
        let mut result: Vec<Option<BlockInfo>> = Vec::new();
        block_ids.into_iter().for_each(|hash| {
            result.push(self.chain().get_block_info(Some(hash)).unwrap());
        });
        async move {
            let _ = self.select_a_peer()?;
            self.err_mocker.random_err().await?;
            Ok(result)
        }
        .boxed()
    }
}

impl SyncFetcher for SyncNodeMocker {}
