// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::sync_metrics::SYNC_METRICS;
use crate::tasks::sync_score_metrics::SYNC_SCORE_METRICS;
use anyhow::{ensure, format_err, Result};
use logger::prelude::*;
use network_api::peer_score::{InverseScore, Score};
use network_api::PeerSelector;
use rand::prelude::SliceRandom;
use starcoin_accumulator::node::AccumulatorStoreType;
use starcoin_accumulator::AccumulatorNode;
use starcoin_crypto::hash::HashValue;
use starcoin_network_rpc_api::{
    gen_client::NetworkRpcClient, BlockBody, GetAccumulatorNodeByNodeHash, GetBlockHeaders,
    GetBlockHeadersByNumber, GetBlockIds, GetTxnsWithHash, RawRpcClient,
};
use starcoin_state_tree::StateNode;
use starcoin_sync_api::SyncTarget;
use starcoin_types::block::Block;
use starcoin_types::peer_info::PeerInfo;
use starcoin_types::transaction::Transaction;
use starcoin_types::{
    block::{BlockHeader, BlockInfo, BlockNumber},
    peer_info::PeerId,
    transaction::TransactionInfo,
};
use std::fmt::Debug;
use std::sync::Arc;
use std::time::Instant;
use thiserror::Error;

#[derive(Clone, Debug, Error)]
pub enum RpcVerifyError {
    #[error("Peer {0:?} return valid rpc response: {1:?}")]
    InvalidResponse(PeerId, String),
}

pub trait RpcVerifier<Req, Resp> {
    /// verify the rpc request and response
    fn verify(&self, peer_id: PeerId, req: Req, resp: Resp) -> Result<Resp, RpcVerifyError>;
}

impl<Req, ReqItem, RespItem, F> RpcVerifier<Req, Vec<RespItem>> for F
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
        resp: Vec<RespItem>,
    ) -> Result<Vec<RespItem>, RpcVerifyError> {
        req.into_iter()
            .zip(resp.into_iter())
            .into_iter()
            .map(|(req_item, resp_item)| {
                if (self)(&req_item, &resp_item) {
                    Ok(resp_item)
                } else {
                    Err(RpcVerifyError::InvalidResponse(
                        peer_id.clone(),
                        format!(
                            "req item: {:?} not match with resp item: {:?}",
                            req_item, resp_item
                        ),
                    ))
                }
            })
            .collect::<Result<Vec<RespItem>, RpcVerifyError>>()
    }
}

static BLOCK_NUMBER_VERIFIER: fn(&BlockNumber, &BlockHeader) -> bool =
    |block_number, block_header| -> bool { *block_number == block_header.number() };

static BLOCK_ID_VERIFIER: fn(&HashValue, &BlockHeader) -> bool =
    |block_hash, block_header| -> bool { *block_hash == block_header.id() };

static BLOCK_BODY_VERIFIER: fn(&HashValue, &BlockBody) -> bool =
    |body_hash, block_body| -> bool { *body_hash == block_body.hash() };

static BLOCK_INFO_VERIFIER: fn(&HashValue, &BlockInfo) -> bool =
    |block_id, block_info| -> bool { *block_id == block_info.block_id };

/// Enhancement RpcClient, for verify rpc response by request and auto select peer.
#[derive(Clone)]
pub struct VerifiedRpcClient {
    peer_selector: PeerSelector,
    client: NetworkRpcClient,
    score_handler: Arc<dyn Score<u32> + 'static>,
}

impl VerifiedRpcClient {
    pub fn new<C>(peer_selector: PeerSelector, raw_rpc_client: C) -> Self
    where
        C: RawRpcClient + Send + Sync + 'static,
    {
        Self::new_with_client(peer_selector, NetworkRpcClient::new(raw_rpc_client))
    }

    pub fn new_with_client(peer_selector: PeerSelector, client: NetworkRpcClient) -> Self {
        Self {
            peer_selector,
            client,
            score_handler: Arc::new(InverseScore::new(100, 60)),
        }
    }

    pub fn selector(&self) -> &PeerSelector {
        &self.peer_selector
    }

    pub fn record(&self, peer: &PeerId, score: i64) {
        self.peer_selector.peer_score(peer, score);
    }

    fn score(&self, time: u32) -> i64 {
        self.score_handler.execute(time)
    }

    pub fn best_peer(&self) -> Option<PeerInfo> {
        if let Some(peers) = self.peer_selector.bests() {
            peers.choose(&mut rand::thread_rng()).cloned()
        } else {
            None
        }
    }

    pub fn select_a_peer(&self) -> Result<PeerId> {
        self.peer_selector
            .select_peer()
            .ok_or_else(|| format_err!("No peers for send request."))
    }

    pub async fn get_txns(
        &self,
        peer_id: Option<PeerId>,
        req: GetTxnsWithHash,
    ) -> Result<(Vec<HashValue>, Vec<Transaction>)> {
        let peer_id = peer_id.unwrap_or(self.select_a_peer()?);
        let data = self.client.get_txns(peer_id.clone(), req.clone()).await?;
        if data.len() == req.len() {
            let mut none_txn_vec = Vec::new();
            let mut verified_txns: Vec<Transaction> = Vec::new();
            for (id, data) in req.ids.into_iter().zip(data.into_iter()) {
                match data {
                    Some(txn) => {
                        ensure!(
                            id == txn.id(),
                            "request txn with id: {} from peer {}, but got txn {:?}",
                            id,
                            peer_id,
                            txn
                        );
                        verified_txns.push(txn);
                    }
                    None => none_txn_vec.push(id),
                }
            }
            Ok((none_txn_vec, verified_txns))
        } else {
            Err(format_err!(
                "Txn len mismatch {:?} : {:?} from peer : {:?}.",
                data.len(),
                req.len(),
                peer_id
            ))
        }
    }

    pub async fn get_txn_infos(
        &self,
        block_id: HashValue,
    ) -> Result<(PeerId, Option<Vec<TransactionInfo>>)> {
        let peer_id = self.select_a_peer()?;
        Ok((
            peer_id.clone(),
            self.client.get_txn_infos(peer_id, block_id).await?,
        ))
    }

    pub async fn get_headers_by_number(
        &self,
        req: GetBlockHeadersByNumber,
    ) -> Result<Vec<BlockHeader>> {
        let peer_id = self.select_a_peer()?;
        let resp: Vec<BlockHeader> = self
            .client
            .get_headers_by_number(peer_id.clone(), req.clone())
            .await?;
        let resp = BLOCK_NUMBER_VERIFIER.verify(peer_id, req, resp)?;
        Ok(resp)
    }

    pub async fn get_headers(
        &self,
        req: GetBlockHeaders,
        number: BlockNumber,
    ) -> Result<(Vec<BlockHeader>, PeerId)> {
        let peer_id = self.select_a_peer()?;
        debug!("rpc select peer {:?}", peer_id);
        let resp = self
            .client
            .get_headers(peer_id.clone(), req.clone())
            .await?;
        let req = req.into_numbers(number);
        let resp = BLOCK_NUMBER_VERIFIER.verify(peer_id.clone(), req, resp)?;
        Ok((resp, peer_id))
    }

    pub async fn get_headers_by_hash(&self, req: Vec<HashValue>) -> Result<Vec<BlockHeader>> {
        let peer_id = self.select_a_peer()?;
        let resp: Vec<BlockHeader> = self
            .client
            .get_headers_by_hash(peer_id.clone(), req.clone())
            .await?;
        let resp = BLOCK_ID_VERIFIER.verify(peer_id, req, resp)?;
        Ok(resp)
    }

    pub async fn get_bodies_by_hash(
        &self,
        req: Vec<HashValue>,
    ) -> Result<(Vec<BlockBody>, PeerId)> {
        let peer_id = self.select_a_peer()?;
        debug!("rpc select peer {}", &peer_id);
        let resp: Vec<BlockBody> = self
            .client
            .get_bodies_by_hash(peer_id.clone(), req.clone())
            .await?;
        let resp = BLOCK_BODY_VERIFIER.verify(peer_id.clone(), req, resp)?;
        Ok((resp, peer_id))
    }

    pub async fn get_block_infos(&self, hashes: Vec<HashValue>) -> Result<Vec<BlockInfo>> {
        self.get_block_infos_from_peer(None, hashes).await
    }

    pub async fn get_block_infos_from_peer(
        &self,
        peer_id: Option<PeerId>,
        req: Vec<HashValue>,
    ) -> Result<Vec<BlockInfo>> {
        let peer_id = match peer_id {
            None => self.select_a_peer()?,
            Some(p) => p,
        };
        let resp = self
            .client
            .get_block_infos(peer_id.clone(), req.clone())
            .await?;
        let resp = BLOCK_INFO_VERIFIER.verify(peer_id, req, resp)?;
        Ok(resp)
    }

    pub async fn get_sync_target(&self) -> Result<SyncTarget> {
        //TODO optimize target selector,
        let best_peers = self
            .peer_selector
            .bests()
            .ok_or_else(|| format_err!("No best peer to request"))?;
        let peer = best_peers
            .choose(&mut rand::thread_rng())
            .ok_or_else(|| format_err!("No peer to request"))?;
        let peer_id = peer.peer_id();
        let header = peer.latest_header();
        let block_id = header.id();

        let block_info = self
            .client
            .get_block_infos(peer_id.clone(), vec![block_id])
            .await?
            .pop()
            .ok_or_else(|| {
                format_err!(
                    "Get block info by id:{} from peer_id:{} return None",
                    block_id,
                    peer_id
                )
            })?;
        ensure!(
            block_id == block_info.block_id,
            "Invalid block info from {}",
            peer_id
        );
        Ok(SyncTarget {
            block_header: header.clone(),
            block_info,
            peers: self.peer_selector.peers(),
        })
    }

    pub async fn get_blocks_by_number(
        &self,
        req: GetBlockHeaders,
        number: BlockNumber,
    ) -> Result<Vec<Block>> {
        let (headers, _) = self.get_headers(req, number).await?;
        let hashs = headers.iter().map(|header| header.id()).collect();
        let (bodies, _) = self.get_bodies_by_hash(hashs).await?;
        //TODO verify headers and bodies' length.
        let blocks: Vec<Block> = headers
            .into_iter()
            .zip(bodies.into_iter())
            .map(|(header, body)| Block::new(header, body))
            .collect();
        Ok(blocks)
    }

    pub async fn get_state_node_by_node_hash(
        &self,
        node_key: HashValue,
    ) -> Result<(PeerId, Option<StateNode>)> {
        let peer_id = self.select_a_peer()?;
        Ok((
            peer_id.clone(),
            self.client
                .get_state_node_by_node_hash(peer_id, node_key)
                .await?,
        ))
    }

    pub async fn get_accumulator_node_by_node_hash(
        &self,
        node_key: HashValue,
        accumulator_type: AccumulatorStoreType,
    ) -> Result<(PeerId, AccumulatorNode)> {
        let peer_id = self.select_a_peer()?;
        if let Some(accumulator_node) = self
            .client
            .get_accumulator_node_by_node_hash(
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
        start_number: BlockNumber,
        reverse: bool,
        max_size: u64,
    ) -> Result<Vec<HashValue>> {
        self.get_block_ids_from_peer(None, start_number, reverse, max_size)
            .await
    }

    pub async fn get_block_ids_from_peer(
        &self,
        peer_id: Option<PeerId>,
        start_number: BlockNumber,
        reverse: bool,
        max_size: u64,
    ) -> Result<Vec<HashValue>> {
        let _timer = SYNC_METRICS
            .sync_get_block_ids_time
            .with_label_values(&["time"])
            .start_timer();
        let peer_id = match peer_id {
            None => self.select_a_peer()?,
            Some(p) => p,
        };
        let request = GetBlockIds {
            start_number,
            reverse,
            max_size,
        };
        self.client.get_block_ids(peer_id, request).await
    }

    pub async fn get_blocks(
        &self,
        ids: Vec<HashValue>,
    ) -> Result<Vec<Option<(Block, Option<PeerId>)>>> {
        let peer_id = self.select_a_peer()?;
        let timer = SYNC_SCORE_METRICS
            .peer_sync_per_time
            .with_label_values(&[&format!("peer-{:?}", peer_id)])
            .start_timer();
        let start_time = Instant::now();
        let blocks: Vec<Option<Block>> =
            self.client.get_blocks(peer_id.clone(), ids.clone()).await?;
        let _ = timer.stop_and_record();
        let time = (Instant::now()
            .saturating_duration_since(start_time)
            .as_millis()) as u32;
        let score = self.score(time);
        self.record(&peer_id, score);
        SYNC_SCORE_METRICS.update_metrics(peer_id.clone(), time, score);
        Ok(ids
            .into_iter()
            .zip(blocks)
            .map(|(id, block)| {
                if let Some(block) = block {
                    let actual_id = block.id();
                    if actual_id != id {
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
            .collect())
    }
}
