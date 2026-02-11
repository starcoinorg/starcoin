// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{format_err, Result};
use network_api::peer_score::{InverseScore, Score};
use network_api::PeerId;
use network_api::PeerInfo;
use network_api::PeerSelector;
use network_api::PeerStrategy;
use starcoin_accumulator::node::AccumulatorStoreType;
use starcoin_accumulator::AccumulatorNode;
use starcoin_crypto::hash::HashValue;
use starcoin_logger::prelude::*;
use starcoin_network_rpc_api::{
    gen_client::NetworkRpcClient, BlockBody, GetAccumulatorNodeByNodeHash, GetBlockHeadersByNumber,
    GetBlockIds, GetTxnsWithHash, RawRpcClient,
};
use starcoin_network_rpc_api::{GetAbsentBlockRequest, GetRangeInLocationRequest, RangeInLocation};
// use starcoin_state_tree::StateNode;  // Unused
use starcoin_types::block::Block;
use starcoin_types::multi_transaction::MultiSignedUserTransaction;
use starcoin_types::{
    block::{BlockHeader, BlockInfo, BlockNumber},
    transaction::{StcTransaction, StcTransactionInfo},
};
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::time::timeout;

use network_p2p_types::{OutboundFailure, RequestFailure};

#[derive(Clone, Debug, Error)]
#[error("Peer {peers:?} return valid rpc response: {msg:?}")]
pub struct RpcVerifyError {
    pub peers: Vec<PeerId>,
    pub msg: String,
}

impl RpcVerifyError {
    pub fn new(peer_id: PeerId, msg: String) -> Self {
        Self {
            peers: vec![peer_id],
            msg,
        }
    }

    pub fn new_with_peers(peers: Vec<PeerId>, msg: String) -> Self {
        Self { peers, msg }
    }
}

pub trait RpcVerifier<Req, Resp> {
    /// verify the rpc request and response
    fn verify(&self, peer_id: PeerId, req: Req, resp: Resp) -> Result<Resp, RpcVerifyError>;
}

impl<Req, ReqItem, RespItem, F> RpcVerifier<Req, Vec<Option<RespItem>>> for F
where
    Req: IntoIterator<Item = ReqItem>,
    ReqItem: Debug,
    RespItem: Debug,
    F: Fn(&ReqItem, &RespItem) -> bool,
{
    fn verify(
        &self,
        peer_id: PeerId,
        req: Req,
        resp: Vec<Option<RespItem>>,
    ) -> Result<Vec<Option<RespItem>>, RpcVerifyError> {
        req.into_iter()
            .zip(resp)
            .map(|(req_item, resp_item)| {
                if let Some(resp_item) = resp_item {
                    if (self)(&req_item, &resp_item) {
                        Ok(Some(resp_item))
                    } else {
                        Err(RpcVerifyError::new(
                            peer_id.clone(),
                            format!(
                                "request item: {:?} not match with resp item: {:?}",
                                req_item, resp_item
                            ),
                        ))
                    }
                } else {
                    Ok(None)
                }
            })
            .collect::<Result<Vec<Option<RespItem>>, RpcVerifyError>>()
    }
}

static G_BLOCK_NUMBER_VERIFIER: fn(&BlockNumber, &BlockHeader) -> bool =
    |block_number, block_header| -> bool { *block_number == block_header.number() };

static G_BLOCK_ID_VERIFIER: fn(&HashValue, &BlockHeader) -> bool =
    |block_hash, block_header| -> bool { *block_hash == block_header.id() };

static G_BLOCK_BODY_VERIFIER: fn(&HashValue, &BlockBody) -> bool =
    |body_hash, block_body| -> bool { *body_hash == block_body.hash() };

static G_BLOCK_INFO_VERIFIER: fn(&HashValue, &BlockInfo) -> bool =
    |block_id, block_info| -> bool { *block_id == block_info.block_id };

#[cfg(not(test))]
const DEFAULT_RPC_RETRY_COUNT: i32 = 20;
#[cfg(test)]
const DEFAULT_RPC_RETRY_COUNT: i32 = 2;

#[cfg(not(test))]
const DEFAULT_RPC_TIMEOUT: Duration = Duration::from_secs(15);
#[cfg(test)]
fn default_rpc_timeout() -> Duration {
    if let Ok(value) = std::env::var("TEST_RPC_TIMEOUT_MS") {
        if let Ok(ms) = value.parse::<u64>() {
            return Duration::from_millis(ms);
        }
    }
    Duration::from_millis(500)
}

struct AdaptiveBlockIdsSizer {
    min: u64,
    max: u64,
    grow_threshold: u64,
    size: AtomicU64,
    success_blocks: AtomicU64,
}

impl AdaptiveBlockIdsSizer {
    fn new(start: u64, min: u64, max: u64, grow_threshold: u64) -> Self {
        let mut initial = start;
        if initial < min {
            initial = min;
        }
        if initial > max {
            initial = max;
        }
        Self {
            min,
            max,
            grow_threshold,
            size: AtomicU64::new(initial),
            success_blocks: AtomicU64::new(0),
        }
    }

    fn clamp_request_size(&self, requested: u64) -> u64 {
        let current = self.size.load(Ordering::Relaxed);
        std::cmp::min(requested, current)
    }

    fn on_timeout(&self) {
        let current = self.size.load(Ordering::Relaxed);
        let mut next = current / 2;
        if next < self.min {
            next = self.min;
        }
        if next != current {
            info!(
                "[sync] get_block_ids batch size reduced on timeout: {} -> {}",
                current, next
            );
            self.size.store(next, Ordering::Relaxed);
        }
        self.success_blocks.store(0, Ordering::Relaxed);
    }

    fn on_success(&self, blocks: u64) {
        if blocks == 0 {
            return;
        }
        let total = self
            .success_blocks
            .fetch_add(blocks, Ordering::Relaxed)
            .saturating_add(blocks);
        if total < self.grow_threshold {
            return;
        }
        let current = self.size.load(Ordering::Relaxed);
        let mut next = current.saturating_mul(2);
        if next > self.max {
            next = self.max;
        }
        if next != current {
            info!(
                "[sync] get_block_ids batch size increased after stable fetch: {} -> {}",
                current, next
            );
            self.size.store(next, Ordering::Relaxed);
        }
        self.success_blocks.store(0, Ordering::Relaxed);
    }
}

fn is_timeout_error(err: &anyhow::Error) -> bool {
    if let Some(req_failure) = err.downcast_ref::<RequestFailure>() {
        return matches!(
            req_failure,
            RequestFailure::Network(OutboundFailure::Timeout)
        );
    }
    // Fallback: match the formatted message from request-response layer.
    err.to_string().contains("Timeout")
}
/// Enhancement RpcClient, for verify rpc response by request and auto select peer.
#[derive(Clone)]
pub struct VerifiedRpcClient {
    peer_selector: PeerSelector,
    client: NetworkRpcClient,
    score_handler: InverseScore,
    max_retry_times: u64,
    rpc_timeout: Duration,
    rpc_retry_count: i32,
    block_ids_sizer: Arc<AdaptiveBlockIdsSizer>,
}

impl VerifiedRpcClient {
    pub fn new<C>(peer_selector: PeerSelector, raw_rpc_client: C, max_retry_times: u64) -> Self
    where
        C: RawRpcClient + Send + Sync + 'static,
    {
        Self::new_with_client(
            peer_selector,
            NetworkRpcClient::new(raw_rpc_client),
            max_retry_times,
        )
    }

    pub fn new_with_client(
        peer_selector: PeerSelector,
        client: NetworkRpcClient,
        max_retry_times: u64,
    ) -> Self {
        let rpc_timeout = {
            #[cfg(not(test))]
            {
                DEFAULT_RPC_TIMEOUT
            }
            #[cfg(test)]
            {
                default_rpc_timeout()
            }
        };
        Self {
            peer_selector,
            client,
            score_handler: InverseScore::new(100, 60),
            max_retry_times,
            rpc_timeout,
            rpc_retry_count: DEFAULT_RPC_RETRY_COUNT,
            block_ids_sizer: Arc::new(AdaptiveBlockIdsSizer::new(1000, 10, 1000, 10_000)),
        }
    }

    pub fn with_rpc_config(mut self, rpc_timeout: Duration, rpc_retry_count: i32) -> Self {
        self.rpc_timeout = rpc_timeout;
        self.rpc_retry_count = rpc_retry_count;
        self
    }

    pub fn switch_strategy(&mut self, strategy: PeerStrategy) {
        self.peer_selector.switch_strategy(strategy)
    }

    pub fn selector(&self) -> &PeerSelector {
        &self.peer_selector
    }

    pub fn record(&self, peer: &PeerId, score: u64) {
        self.peer_selector.peer_score(peer, score);
    }

    fn score(&self, time: u32) -> u64 {
        self.score_handler.execute(time)
    }

    pub fn best_peer(&self) -> Option<PeerInfo> {
        self.peer_selector.best()
    }

    pub fn select_a_peer(&self) -> Result<PeerId> {
        self.peer_selector
            .select_peer()
            .ok_or_else(|| format_err!("No peers for send request."))
    }

    async fn get_txns_with_hash_from_pool_inner(
        &self,
        peer_id: PeerId,
        req: GetTxnsWithHash,
    ) -> Result<Vec<Option<MultiSignedUserTransaction>>> {
        let mut count = 0;
        while count < self.rpc_retry_count {
            match self
                .client
                .get_txns_with_hash_from_pool(peer_id.clone(), req.clone())
                .await
            {
                Ok(result) => return Ok(result),
                Err(e) => {
                    count = count.saturating_add(1);
                    if count == self.rpc_retry_count {
                        return Err(RpcVerifyError::new(
                            peer_id.clone(),
                            format!(
                                "failed to get txns with hash from pool from peer : {:?}. error: {:?}",
                                peer_id, e
                            ),
                        )
                        .into());
                    }
                    continue;
                }
            }
        }
        Err(RpcVerifyError::new(
            peer_id.clone(),
            format!(
                "failed to get txns with hash from pool from peer : {:?}.",
                peer_id,
            ),
        )
        .into())
    }

    pub async fn get_txns_with_hash_from_pool(
        &self,
        peer_id: Option<PeerId>,
        req: GetTxnsWithHash,
    ) -> Result<(Vec<HashValue>, Vec<MultiSignedUserTransaction>)> {
        let peer_id = if let Some(peer) = peer_id {
            peer
        } else {
            self.select_a_peer()?
        };
        let data = self
            .get_txns_with_hash_from_pool_inner(peer_id.clone(), req.clone())
            .await?;
        if data.len() == req.len() {
            let mut none_txn_vec = Vec::new();
            let mut verified_txns: Vec<MultiSignedUserTransaction> = Vec::new();
            for (id, data) in req.ids.into_iter().zip(data.into_iter()) {
                match data {
                    Some(txn) => {
                        if id != txn.id() {
                            return Err(RpcVerifyError::new(
                                peer_id.clone(),
                                format!(
                                    "request txn with id: {} from peer {}, but got txn {:?}",
                                    id, peer_id, txn
                                ),
                            )
                            .into());
                        }
                        verified_txns.push(txn);
                    }
                    None => none_txn_vec.push(id),
                }
            }
            Ok((none_txn_vec, verified_txns))
        } else {
            Err(RpcVerifyError::new(
                peer_id.clone(),
                format!(
                    "Txn len mismatch {:?} : {:?} from peer : {:?}.",
                    data.len(),
                    req.len(),
                    peer_id
                ),
            )
            .into())
        }
    }

    async fn get_txns_inner(
        &self,
        peer_id: PeerId,
        req: GetTxnsWithHash,
    ) -> Result<Vec<Option<StcTransaction>>> {
        let mut count = 0;
        while count < self.rpc_retry_count {
            match self.client.get_txns(peer_id.clone(), req.clone()).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    count = count.saturating_add(1);
                    if count == self.rpc_retry_count {
                        return Err(RpcVerifyError::new(
                            peer_id.clone(),
                            format!(
                                "failed to get txns from peer : {:?}. error: {:?}",
                                peer_id, e
                            ),
                        )
                        .into());
                    }
                    continue;
                }
            }
        }
        Err(RpcVerifyError::new(
            peer_id.clone(),
            format!("failed to get txns from peer : {:?}.", peer_id,),
        )
        .into())
    }

    pub async fn get_txns(
        &self,
        peer_id: Option<PeerId>,
        req: GetTxnsWithHash,
    ) -> Result<(Vec<HashValue>, Vec<StcTransaction>)> {
        let peer_id = peer_id.unwrap_or(self.select_a_peer()?);
        let data = self.get_txns_inner(peer_id.clone(), req.clone()).await?;
        if data.len() == req.len() {
            let mut none_txn_vec = Vec::new();
            let mut verified_txns: Vec<StcTransaction> = Vec::new();
            for (id, data) in req.ids.into_iter().zip(data.into_iter()) {
                match data {
                    Some(txn) => {
                        if id != txn.id() {
                            return Err(RpcVerifyError::new(
                                peer_id.clone(),
                                format!(
                                    "request txn with id: {} from peer {}, but got txn {:?}",
                                    id, peer_id, txn
                                ),
                            )
                            .into());
                        }
                        verified_txns.push(txn);
                    }
                    None => none_txn_vec.push(id),
                }
            }
            Ok((none_txn_vec, verified_txns))
        } else {
            Err(RpcVerifyError::new(
                peer_id.clone(),
                format!(
                    "Txn len mismatch {:?} : {:?} from peer : {:?}.",
                    data.len(),
                    req.len(),
                    peer_id
                ),
            )
            .into())
        }
    }

    pub async fn get_txn_infos(
        &self,
        block_id: HashValue,
    ) -> Result<(PeerId, Option<Vec<StcTransactionInfo>>)> {
        let peer_id = self.select_a_peer()?;
        let mut count = 0;
        while count < self.rpc_retry_count {
            match self.client.get_txn_infos(peer_id.clone(), block_id).await {
                Ok(result) => return Ok((peer_id, result)),
                Err(e) => {
                    count = count.saturating_add(1);
                    if count == self.rpc_retry_count {
                        return Err(RpcVerifyError::new(
                            peer_id.clone(),
                            format!(
                                "failed to get txn infos from peer : {:?}. error: {:?}",
                                peer_id, e
                            ),
                        )
                        .into());
                    }
                    continue;
                }
            }
        }
        Err(RpcVerifyError::new(
            peer_id.clone(),
            format!("failed to get txn infos from peer : {:?}.", peer_id,),
        )
        .into())
    }

    pub async fn get_headers_by_number(
        &self,
        req: GetBlockHeadersByNumber,
    ) -> Result<Vec<Option<BlockHeader>>> {
        let peer_id = self.select_a_peer()?;
        let mut count = 0;
        while count < self.rpc_retry_count {
            match self
                .client
                .get_headers_by_number(peer_id.clone(), req.clone())
                .await
            {
                Ok(result) => {
                    return Ok(G_BLOCK_NUMBER_VERIFIER.verify(peer_id, req, result)?);
                }
                Err(e) => {
                    count = count.saturating_add(1);
                    if count == self.rpc_retry_count {
                        return Err(RpcVerifyError::new(
                            peer_id.clone(),
                            format!(
                                "failed to get block headers from peer : {:?}., error: {:?}",
                                peer_id, e
                            ),
                        )
                        .into());
                    }
                    continue;
                }
            }
        }
        Err(RpcVerifyError::new(
            peer_id.clone(),
            format!("failed to get block headers from peer : {:?}.", peer_id,),
        )
        .into())
    }

    pub async fn get_headers_by_hash(
        &self,
        req: Vec<HashValue>,
    ) -> Result<Vec<Option<BlockHeader>>> {
        let peer_id = self.select_a_peer()?;
        let mut count = 0;
        while count < self.rpc_retry_count {
            match self
                .client
                .get_headers_by_hash(peer_id.clone(), req.clone())
                .await
            {
                Ok(result) => {
                    return Ok(G_BLOCK_ID_VERIFIER.verify(peer_id, req, result)?);
                }
                Err(e) => {
                    count = count.saturating_add(1);
                    if count == self.rpc_retry_count {
                        return Err(RpcVerifyError::new(
                            peer_id.clone(),
                            format!(
                                "failed to get block headers from peer : {:?}., error: {:?}",
                                peer_id, e
                            ),
                        )
                        .into());
                    }
                    continue;
                }
            }
        }
        Err(RpcVerifyError::new(
            peer_id.clone(),
            format!("failed to get block headers from peer : {:?}.", peer_id,),
        )
        .into())
    }

    pub async fn get_bodies_by_hash(
        &self,
        req: Vec<HashValue>,
    ) -> Result<(Vec<Option<BlockBody>>, PeerId)> {
        let peer_id = self.select_a_peer()?;
        debug!("rpc select peer {}", &peer_id);
        let mut count = 0;
        while count < self.rpc_retry_count {
            match self
                .client
                .get_bodies_by_hash(peer_id.clone(), req.clone())
                .await
            {
                Ok(result) => {
                    return Ok((
                        G_BLOCK_BODY_VERIFIER.verify(peer_id.clone(), req, result)?,
                        peer_id,
                    ));
                }
                Err(e) => {
                    count = count.saturating_add(1);
                    if count == self.rpc_retry_count {
                        return Err(RpcVerifyError::new(
                            peer_id.clone(),
                            format!(
                                "failed to get block bodies from peer : {:?}. error: {:?}",
                                peer_id, e
                            ),
                        )
                        .into());
                    }
                    continue;
                }
            }
        }
        Err(RpcVerifyError::new(
            peer_id.clone(),
            format!("failed to get block bodies from peer : {:?}.", peer_id,),
        )
        .into())
    }

    pub async fn get_block_infos(&self, hashes: Vec<HashValue>) -> Result<Vec<Option<BlockInfo>>> {
        self.get_block_infos_from_peer(None, hashes).await
    }

    pub async fn get_block_infos_from_peer(
        &self,
        peer_id: Option<PeerId>,
        req: Vec<HashValue>,
    ) -> Result<Vec<Option<BlockInfo>>> {
        let peer_id = match peer_id {
            None => self.select_a_peer()?,
            Some(p) => p,
        };

        let mut count = 0;
        while count < self.rpc_retry_count {
            match self
                .client
                .get_block_infos(peer_id.clone(), req.clone())
                .await
            {
                Ok(result) => {
                    return Ok(G_BLOCK_INFO_VERIFIER.verify(peer_id, req, result)?);
                }
                Err(e) => {
                    count = count.saturating_add(1);
                    if count == self.rpc_retry_count {
                        return Err(RpcVerifyError::new(
                            peer_id.clone(),
                            format!(
                                "failed to get block infos from peer : {:?}. error: {:?}",
                                peer_id, e
                            ),
                        )
                        .into());
                    }
                    continue;
                }
            }
        }
        Err(RpcVerifyError::new(
            peer_id.clone(),
            format!("failed to get block infos from peer : {:?}.", peer_id,),
        )
        .into())
    }

    // TODO: This function seems to be unused and the RPC method doesn't exist
    // pub async fn get_state_node_by_node_hash(
    //     &self,
    //     node_key: HashValue,
    // ) -> Result<(PeerId, Option<StateNode>)> {
    //     let peer_id = self.select_a_peer()?;
    //     let mut count = 0;
    //     while count < self.rpc_retry_count {
    //         match self
    //             .client
    //             .get_state_node_by_node_hash(peer_id.clone(), node_key)
    //             .await
    //         {
    //             Ok(result) => return Ok((peer_id, result)),
    //             Err(e) => {
    //                 count = count.saturating_add(1);
    //                 if count == self.rpc_retry_count {
    //                     return Err(RpcVerifyError::new(
    //                         peer_id.clone(),
    //                         format!(
    //                             "failed to get state node by node hash from peer : {:?}. error: {:?}",
    //                             peer_id, e
    //                         ),
    //                     )
    //                     .into());
    //                 }
    //                 continue;
    //             }
    //         }
    //     }
    //     Err(RpcVerifyError::new(
    //         peer_id.clone(),
    //         format!(
    //             "failed to get state node by node hash from peer : {:?}",
    //             peer_id,
    //         ),
    //     )
    //     .into())
    // }

    async fn get_accumulator_node_by_node_hash_inner(
        &self,
        peer_id: PeerId,
        req: GetAccumulatorNodeByNodeHash,
    ) -> Result<Option<AccumulatorNode>> {
        let mut count = 0;
        while count < self.rpc_retry_count {
            match self
                .client
                .get_accumulator_node_by_node_hash(peer_id.clone(), req.clone())
                .await
            {
                Ok(result) => return Ok(result),
                Err(e) => {
                    count = count.saturating_add(1);
                    if count == self.rpc_retry_count {
                        return Err(RpcVerifyError::new(
                            peer_id.clone(),
                            format!("failed to get accumulator node by node hash inner from peer : {:?}. error: {:?}", peer_id, e),
                        )
                        .into());
                    }
                    continue;
                }
            }
        }
        Err(RpcVerifyError::new(
            peer_id.clone(),
            format!(
                "failed to get accumulator node by node hash inner from peer : {:?}.",
                peer_id
            ),
        )
        .into())
    }

    pub async fn get_accumulator_node_by_node_hash(
        &self,
        node_key: HashValue,
        accumulator_type: AccumulatorStoreType,
    ) -> Result<(PeerId, AccumulatorNode)> {
        let peer_id = self.select_a_peer()?;
        if let Some(accumulator_node) = self
            .get_accumulator_node_by_node_hash_inner(
                peer_id.clone(),
                GetAccumulatorNodeByNodeHash {
                    node_hash: node_key,
                    accumulator_storage_type: accumulator_type,
                },
            )
            .await?
        {
            let accumulator_node_id = accumulator_node.hash();
            if node_key == accumulator_node_id {
                Ok((peer_id, accumulator_node))
            } else {
                Err(format_err!(
                    "Accumulator node hash {:?} and node key {:?} mismatch.",
                    accumulator_node_id,
                    node_key
                ))
            }
        } else {
            Err(format_err!(
                "Accumulator node is none by node key {:?}.",
                node_key
            ))
        }
    }

    pub async fn get_block_ids(
        &self,
        peer_id: Option<PeerId>,
        start_number: BlockNumber,
        reverse: bool,
        max_size: u64,
    ) -> Result<Vec<HashValue>> {
        let peer_id = match peer_id {
            None => self.select_a_peer()?,
            Some(p) => p,
        };
        let requested_max = max_size;
        let mut request = GetBlockIds {
            start_number,
            reverse,
            max_size: self.block_ids_sizer.clamp_request_size(requested_max),
        };
        let mut count = 0;
        while count < self.rpc_retry_count {
            match self
                .client
                .get_block_ids(peer_id.clone(), request.clone())
                .await
            {
                Ok(result) => {
                    self.block_ids_sizer.on_success(result.len() as u64);
                    return Ok(result);
                }
                Err(e) => {
                    if is_timeout_error(&e) {
                        self.block_ids_sizer.on_timeout();
                        request.max_size = self.block_ids_sizer.clamp_request_size(requested_max);
                    }
                    count = count.saturating_add(1);
                    if count == self.rpc_retry_count {
                        return Err(RpcVerifyError::new(
                            peer_id.clone(),
                            format!(
                                "failed to get block ids from peer : {:?}. error: {:?}",
                                peer_id, e
                            ),
                        )
                        .into());
                    }
                    continue;
                }
            }
        }
        Err(RpcVerifyError::new(
            peer_id.clone(),
            format!("failed to get block ids from peer : {:?}.", peer_id),
        )
        .into())
    }

    pub async fn get_block_headers_by_hash(
        &self,
        ids: Vec<HashValue>,
    ) -> Result<Vec<(HashValue, Option<BlockHeader>)>> {
        let mut count = 0;
        let peer_id = self.select_a_peer()?;
        while count < self.rpc_retry_count {
            match timeout(
                self.rpc_timeout,
                self.client
                    .get_headers_by_hash(peer_id.clone(), ids.clone()),
            )
            .await
            {
                Ok(Ok(result)) => return Ok(ids.into_iter().zip(result.into_iter()).collect()),
                Ok(Err(e)) => {
                    count = count.saturating_add(1);
                    if count == self.rpc_retry_count {
                        return Err(RpcVerifyError::new(
                            peer_id.clone(),
                            format!(
                                "failed to get block headers from peer : {:?}., error: {:?}",
                                peer_id, e
                            ),
                        )
                        .into());
                    }
                    continue;
                }
                Err(_) => {
                    count = count.saturating_add(1);
                    if count == self.rpc_retry_count {
                        return Err(RpcVerifyError::new(
                            peer_id.clone(),
                            format!(
                                "failed to get block headers from peer : {:?}., error: timeout",
                                peer_id
                            ),
                        )
                        .into());
                    }
                    continue;
                }
            }
        }
        Err(RpcVerifyError::new(
            peer_id.clone(),
            format!("failed to get block headers from peer : {:?}.", peer_id),
        )
        .into())
    }

    async fn get_blocks_inner(
        &self,
        peer_id: PeerId,
        ids: Vec<HashValue>,
    ) -> Result<Vec<Option<Block>>> {
        let mut count = 0;
        while count < self.rpc_retry_count {
            match timeout(
                self.rpc_timeout,
                self.client.get_blocks(peer_id.clone(), ids.clone()),
            )
            .await
            {
                Ok(Ok(result)) => return Ok(result),
                Ok(Err(e)) => {
                    count = count.saturating_add(1);
                    if count == self.rpc_retry_count {
                        return Err(RpcVerifyError::new(
                            peer_id.clone(),
                            format!(
                                "failed to get blocks v1 from peer : {:?}. error: {:?}",
                                peer_id, e
                            ),
                        )
                        .into());
                    }
                    continue;
                }
                Err(_) => {
                    count = count.saturating_add(1);
                    if count == self.rpc_retry_count {
                        return Err(RpcVerifyError::new(
                            peer_id.clone(),
                            format!(
                                "failed to get blocks v1 from peer : {:?}. error: timeout",
                                peer_id
                            ),
                        )
                        .into());
                    }
                    continue;
                }
            }
        }
        Err(RpcVerifyError::new(
            peer_id.clone(),
            format!("failed to get blocks from peer : {:?}.", peer_id),
        )
        .into())
    }

    pub async fn get_block_diligently(
        &self,
        ids: Vec<HashValue>,
    ) -> Result<Vec<Option<(Block, Option<PeerId>)>>> {
        let peer_infos = self
            .peer_selector
            .bests(0.into())
            .ok_or_else(|| format_err!("No peers for send request."))?;

        let mut result: HashMap<HashValue, (Block, PeerId)> = HashMap::new();
        let mut waiting_list = ids.clone();
        for peer_info in peer_infos {
            let blocks = match self
                .get_blocks_inner(peer_info.peer_id(), waiting_list.clone())
                .await
            {
                Ok(blocks) => blocks,
                Err(err) => {
                    warn!("get blocks failed:{}, call get blocks legacy", err);
                    vec![]
                }
            };
            if blocks.is_empty() {
                continue;
            }

            let rpc_result = waiting_list
                .into_iter()
                .zip(blocks)
                .map(|(id, block)| {
                    if let Some(block) = block {
                        let actual_id = block.id();
                        if actual_id != id {
                            warn!(
                                "Get block by id: {:?} from peer: {:?}, but got block: {:?}",
                                id,
                                peer_info.peer_id(),
                                actual_id
                            );
                            (id, None)
                        } else {
                            (id, Some(block))
                        }
                    } else {
                        (id, None)
                    }
                })
                .collect::<Vec<_>>();

            waiting_list = vec![];

            result.extend(
                rpc_result
                    .iter()
                    .filter(|(id, block)| {
                        if block.is_none() {
                            waiting_list.push(*id);
                        }
                        block.is_some()
                    })
                    .cloned()
                    .map(|(id, block)| {
                        (
                            id,
                            (
                                block.expect("block should not be none"),
                                peer_info.peer_id(),
                            ),
                        )
                    })
                    .collect::<Vec<(HashValue, (Block, PeerId))>>(),
            );

            if waiting_list.is_empty() {
                break;
            }
        }

        Ok(ids
            .into_iter()
            .map(|id| {
                result.get(&id).map(|block_and_peerid| {
                    (block_and_peerid.0.clone(), Some(block_and_peerid.1.clone()))
                })
            })
            .collect())
    }

    pub async fn get_blocks(
        &self,
        ids: Vec<HashValue>,
    ) -> Result<Vec<Option<(Block, Option<PeerId>)>>> {
        let mut peer_iter = self.peer_selector.peer_iterator();
        let max_retries = self.max_retry_times;
        let mut attempts: u64 = 0;
        let mut last_error = None;

        while attempts < max_retries && peer_iter.remaining() > 0 {
            if let Some(peer_id) = peer_iter.next() {
                attempts = attempts.saturating_add(1);
                debug!(
                    "get_blocks: attempting peer {} (attempt {}/{}, {} peers remaining)",
                    peer_id,
                    attempts,
                    max_retries,
                    peer_iter.remaining()
                );

                let start_time = Instant::now();
                match self.get_blocks_inner(peer_id.clone(), ids.clone()).await {
                    Ok(blocks) => {
                        let time = (Instant::now()
                            .saturating_duration_since(start_time)
                            .as_millis()) as u32;
                        let score = self.score(time);
                        self.record(&peer_id, score);
                        return Ok(ids
                            .iter()
                            .zip(blocks)
                            .map(|(id, block)| {
                                if let Some(block) = block {
                                    let actual_id = block.id();
                                    if actual_id != *id {
                                        warn!(
                                            "Get block by id: {:?} from peer: {:?}, but got block: {:?}",
                                            id, peer_id, actual_id
                                        );
                                        None
                                    } else {
                                        Some((block, Some(peer_id.clone())))
                                    }
                                } else {
                                    None
                                }
                            })
                            .collect());
                    }
                    Err(err) => {
                        warn!(
                            "Failed to get blocks from peer {} (attempt {}/{}): {:?}, trying next peer",
                            peer_id, attempts, max_retries, err
                        );
                        self.record(&peer_id, 0);
                        last_error = Some(err);
                    }
                }
            } else {
                break;
            }
        }

        Err(last_error.unwrap_or_else(|| {
            format_err!(
                "Failed to get blocks after {} attempts, no peers available",
                attempts
            )
        }))
    }

    pub async fn get_absent_blocks(
        &self,
        req: Vec<HashValue>,
        exp: u64,
    ) -> Result<Vec<(Block, Option<PeerId>)>> {
        let mut count = 0;
        let peer_id = self.select_a_peer()?;
        while count < self.rpc_retry_count {
            match timeout(
                self.rpc_timeout,
                self.client.get_absent_blocks(
                    peer_id.clone(),
                    GetAbsentBlockRequest {
                        absent_id: req.clone(),
                        exp,
                    },
                ),
            )
            .await
            {
                Ok(Ok(result)) => {
                    return Ok(result
                        .absent_blocks
                        .into_iter()
                        .map(|block| (block, Some(peer_id.clone())))
                        .collect())
                }
                Ok(Err(e)) => {
                    count = count.saturating_add(1);
                    if count == self.rpc_retry_count {
                        return Err(RpcVerifyError::new(
                            peer_id.clone(),
                            format!(
                                "failed to get absent blocks from peer : {:?}. error: {:?}",
                                peer_id, e
                            ),
                        )
                        .into());
                    }
                    continue;
                }
                Err(_) => {
                    count = count.saturating_add(1);
                    if count == self.rpc_retry_count {
                        return Err(RpcVerifyError::new(
                            peer_id.clone(),
                            format!(
                                "failed to get absent blocks from peer : {:?}. error: timeout",
                                peer_id
                            ),
                        )
                        .into());
                    }
                    continue;
                }
            }
        }
        Err(RpcVerifyError::new(
            peer_id.clone(),
            format!("failed to get absent blocks from peer : {:?}.", peer_id),
        )
        .into())
    }

    pub async fn get_dag_block_children(&self, req: Vec<HashValue>) -> Result<Vec<HashValue>> {
        let mut count = 0;
        let peer_id = self.select_a_peer()?;
        while count < self.rpc_retry_count {
            match self
                .client
                .get_dag_block_children(peer_id.clone(), req.clone())
                .await
            {
                Ok(result) => return Ok(result),
                Err(_) => {
                    count = count.saturating_add(1);
                    continue;
                }
            }
        }
        Err(RpcVerifyError::new(
            peer_id.clone(),
            format!(
                "failed to get dag block children from peer : {:?}.",
                peer_id
            ),
        )
        .into())
    }

    pub async fn fetch_range_locate(
        &self,
        peer: Option<PeerId>,
        start_id: HashValue,
        end_id: Option<HashValue>,
    ) -> Result<RangeInLocation> {
        let peer_id = if let Some(peer) = peer {
            peer
        } else {
            self.select_a_peer()?
        };

        let req = GetRangeInLocationRequest { start_id, end_id };

        let mut count = 0;
        while count < self.rpc_retry_count {
            match self
                .client
                .get_range_in_location(peer_id.clone(), req.clone())
                .await
            {
                Ok(result) => {
                    return Ok(result.range);
                }
                Err(e) => {
                    count = count.saturating_add(1);
                    if count == self.rpc_retry_count {
                        return Err(RpcVerifyError::new(
                            peer_id.clone(),
                            format!(
                                "failed to get range in location from peer: {:?}., error: {:?}",
                                peer_id, e
                            ),
                        )
                        .into());
                    }
                    continue;
                }
            }
        }
        Err(RpcVerifyError::new(
            peer_id.clone(),
            format!("failed to get range in location from peer : {:?}.", peer_id,),
        )
        .into())
    }
}
