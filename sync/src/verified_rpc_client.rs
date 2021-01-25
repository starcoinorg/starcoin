// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::sync_metrics::SYNC_METRICS;
use crate::tasks::sync_score_metrics::SYNC_SCORE_METRICS;
use anyhow::{ensure, format_err, Result};
use logger::prelude::*;
use network::get_unix_ts_as_millis;
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
use std::collections::HashSet;
use std::hash::Hash;
use std::sync::Arc;

pub trait RpcVerify<C: Clone> {
    fn filter<T, F>(&mut self, rpc_data: Vec<T>, hash_fun: F) -> Vec<T>
    where
        F: Fn(&T) -> C;
}

pub struct RpcEntryVerify<C: Clone + Eq + Hash> {
    entries: HashSet<C>,
}

impl<C: Clone + Eq + Hash> RpcVerify<C> for RpcEntryVerify<C> {
    fn filter<T, F>(&mut self, rpc_data: Vec<T>, hash_fun: F) -> Vec<T>
    where
        F: Fn(&T) -> C,
    {
        let mut rpc_data = rpc_data;
        let mut dirty_data = Vec::new();
        for data in rpc_data.as_slice().iter() {
            let hash = hash_fun(data);
            if !self.entries.contains(&hash) {
                dirty_data.push(hash);
            }
        }

        if !dirty_data.is_empty() {
            for hash in dirty_data.as_slice().iter() {
                rpc_data.retain(|d| hash_fun(d) != *hash);
            }
        }

        rpc_data
    }
}

impl From<&Vec<HashValue>> for RpcEntryVerify<HashValue> {
    fn from(data: &Vec<HashValue>) -> Self {
        let mut entries = HashSet::new();
        for hash in data.iter() {
            entries.insert(*hash);
        }
        Self { entries }
    }
}

impl From<&Vec<BlockNumber>> for RpcEntryVerify<BlockNumber> {
    fn from(data: &Vec<BlockNumber>) -> Self {
        let mut entries = HashSet::new();
        for number in data.iter() {
            entries.insert(*number);
        }
        Self { entries }
    }
}

impl From<&GetTxnsWithHash> for RpcEntryVerify<HashValue> {
    fn from(data: &GetTxnsWithHash) -> Self {
        let mut entries = HashSet::new();
        for hash in data.clone().ids.into_iter() {
            entries.insert(hash);
        }
        Self { entries }
    }
}

impl From<&GetBlockHeadersByNumber> for RpcEntryVerify<BlockNumber> {
    fn from(data: &GetBlockHeadersByNumber) -> Self {
        let numbers: Vec<BlockNumber> = data.clone().into();
        let mut entries = HashSet::new();
        for number in numbers.into_iter() {
            entries.insert(number);
        }
        Self { entries }
    }
}

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
        req: GetTxnsWithHash,
    ) -> Result<(Vec<HashValue>, Vec<Transaction>)> {
        let peer_id = self.select_a_peer()?;
        let data = self.client.get_txns(peer_id.clone(), req.clone()).await?;
        if data.len() == req.len() {
            let mut none_txn_vec = Vec::new();
            let mut verified_txns: Vec<Transaction> = Vec::new();
            for (id, data) in req.ids.into_iter().zip(data.into_iter()) {
                if data.is_some() {
                    let txn = data.expect("txn is none.");
                    if id == txn.id() {
                        verified_txns.push(txn);
                        continue;
                    }
                }
                none_txn_vec.push(id);
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
        let mut verify_condition: RpcEntryVerify<BlockNumber> = (&req).into();
        let data = self.client.get_headers_by_number(peer_id, req).await?;
        let verified_headers =
            verify_condition.filter(data, |header| -> BlockNumber { header.number() });
        Ok(verified_headers)
    }

    pub async fn get_headers(
        &self,
        req: GetBlockHeaders,
        number: BlockNumber,
    ) -> Result<(Vec<BlockHeader>, PeerId)> {
        let peer_id = self.select_a_peer()?;
        debug!("rpc select peer {:?}", peer_id);
        let mut verify_condition: RpcEntryVerify<BlockNumber> =
            (&req.clone().into_numbers(number)).into();
        let data = self.client.get_headers(peer_id.clone(), req).await?;
        let verified_headers =
            verify_condition.filter(data, |header| -> BlockNumber { header.number() });
        //TODO remove peer_id from result
        Ok((verified_headers, peer_id))
    }

    pub async fn get_headers_by_hash(&self, hashes: Vec<HashValue>) -> Result<Vec<BlockHeader>> {
        let peer_id = self.select_a_peer()?;
        let mut verify_condition: RpcEntryVerify<HashValue> = (&hashes).into();
        let data: Vec<BlockHeader> = self.client.get_headers_by_hash(peer_id, hashes).await?;
        let verified_headers = verify_condition.filter(data, |header| -> HashValue { header.id() });
        Ok(verified_headers)
    }

    pub async fn get_bodies_by_hash(
        &self,
        hashes: Vec<HashValue>,
    ) -> Result<(Vec<BlockBody>, PeerId)> {
        let peer_id = self.select_a_peer()?;
        debug!("rpc select peer {}", &peer_id);
        let mut verify_condition: RpcEntryVerify<HashValue> = (&hashes).into();
        let data: Vec<BlockBody> = self
            .client
            .get_bodies_by_hash(peer_id.clone(), hashes)
            .await?;
        let verified_bodies = verify_condition.filter(data, |body| -> HashValue { body.id() });
        Ok((verified_bodies, peer_id))
    }

    pub async fn get_block_infos(&self, hashes: Vec<HashValue>) -> Result<Vec<BlockInfo>> {
        self.get_block_infos_from_peer(None, hashes).await
    }

    pub async fn get_block_infos_from_peer(
        &self,
        peer_id: Option<PeerId>,
        hashes: Vec<HashValue>,
    ) -> Result<Vec<BlockInfo>> {
        let peer_id = match peer_id {
            None => self.select_a_peer()?,
            Some(p) => p,
        };
        let mut verify_condition: RpcEntryVerify<HashValue> = (&hashes).into();
        let data = self.client.get_block_infos(peer_id, hashes).await?;
        let verified_infos =
            verify_condition.filter(data, |info| -> HashValue { *info.block_id() });
        Ok(verified_infos)
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
        let start_time = get_unix_ts_as_millis();
        let blocks: Vec<Option<Block>> =
            self.client.get_blocks(peer_id.clone(), ids.clone()).await?;
        let _ = timer.stop_and_record();
        let time = (get_unix_ts_as_millis() - start_time) as u32;
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
