// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::tasks::SyncTarget;
use anyhow::{ensure, format_err, Result};
use logger::prelude::*;
use network_api::PeerSelector;
use starcoin_accumulator::node::AccumulatorStoreType;
use starcoin_accumulator::AccumulatorNode;
use starcoin_crypto::hash::HashValue;
use starcoin_crypto::hash::PlainCryptoHash;
use starcoin_network_rpc_api::{
    gen_client::NetworkRpcClient, BlockBody, GetAccumulatorNodeByNodeHash, GetBlockHeaders,
    GetBlockHeadersByNumber, GetBlockIds, GetTxns, RawRpcClient, TransactionsData,
};
use starcoin_state_tree::StateNode;
use starcoin_types::block::Block;
use starcoin_types::peer_info::PeerInfo;
use starcoin_types::{
    block::{BlockHeader, BlockInfo, BlockNumber},
    peer_info::PeerId,
    transaction::TransactionInfo,
};
use std::collections::HashSet;
use std::hash::Hash;

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

impl From<&GetTxns> for RpcEntryVerify<HashValue> {
    fn from(data: &GetTxns) -> Self {
        let mut entries = HashSet::new();
        if let Some(ids) = data.clone().ids {
            for hash in ids.into_iter() {
                entries.insert(hash);
            }
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
}

impl VerifiedRpcClient {
    pub fn new<C>(peer_selector: PeerSelector, raw_rpc_client: C) -> Self
    where
        C: RawRpcClient + Send + Sync + 'static,
    {
        Self {
            peer_selector,
            client: NetworkRpcClient::new(raw_rpc_client),
        }
    }

    pub fn new_with_client(peer_selector: PeerSelector, client: NetworkRpcClient) -> Self {
        Self {
            peer_selector,
            client,
        }
    }

    pub fn best_peer(&self) -> Option<PeerInfo> {
        self.peer_selector.bests().random().cloned()
    }

    pub fn random_peer(&self) -> Result<PeerId> {
        self.peer_selector
            .random_peer_id()
            .ok_or_else(|| format_err!("No peers for send request."))
    }

    pub async fn get_txns(&self, req: GetTxns) -> Result<TransactionsData> {
        let peer_id = self.random_peer()?;
        let data = self.client.get_txns_from_pool(peer_id, req.clone()).await?;
        if req.ids.is_some() {
            let mut verify_condition: RpcEntryVerify<HashValue> = (&req).into();
            let verified_txns = verify_condition
                .filter((*data.get_txns()).to_vec(), |txn| -> HashValue {
                    txn.crypto_hash()
                });
            Ok(TransactionsData {
                txns: verified_txns,
            })
        } else {
            Ok(data)
        }
    }

    pub async fn get_txn_infos(
        &self,
        block_id: HashValue,
    ) -> Result<(PeerId, Option<Vec<TransactionInfo>>)> {
        let peer_id = self.random_peer()?;
        Ok((
            peer_id.clone(),
            self.client.get_txn_infos(peer_id, block_id).await?,
        ))
    }

    pub async fn get_headers_by_number(
        &self,
        req: GetBlockHeadersByNumber,
    ) -> Result<Vec<BlockHeader>> {
        let peer_id = self.random_peer()?;
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
        let peer_id = self.random_peer()?;
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
        let peer_id = self.random_peer()?;
        let mut verify_condition: RpcEntryVerify<HashValue> = (&hashes).into();
        let data: Vec<BlockHeader> = self.client.get_headers_by_hash(peer_id, hashes).await?;
        let verified_headers = verify_condition.filter(data, |header| -> HashValue { header.id() });
        Ok(verified_headers)
    }

    pub async fn get_bodies_by_hash(
        &self,
        hashes: Vec<HashValue>,
    ) -> Result<(Vec<BlockBody>, PeerId)> {
        let peer_id = self.random_peer()?;
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
        let peer_id = self.random_peer()?;
        let mut verify_condition: RpcEntryVerify<HashValue> = (&hashes).into();
        let data = self.client.get_block_infos(peer_id, hashes).await?;
        let verified_infos =
            verify_condition.filter(data, |info| -> HashValue { *info.block_id() });
        Ok(verified_infos)
    }

    pub async fn get_sync_target(&self) -> Result<SyncTarget> {
        //TODO optimize target selector,
        let selector = self.peer_selector.bests();
        let peer = selector
            .random()
            .ok_or_else(|| format_err!("No peer to request"))?;
        let peer_id = peer.get_peer_id();
        let header = peer.get_latest_header();
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
            peers: selector.cloned(),
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
    ) -> Result<(PeerId, StateNode)> {
        let peer_id = self.random_peer()?;
        if let Some(state_node) = self
            .client
            .get_state_node_by_node_hash(peer_id.clone(), node_key)
            .await?
        {
            let state_node_id = state_node.inner().hash();
            if node_key == state_node_id {
                Ok((peer_id, state_node))
            } else {
                Err(format_err!(
                    "State node hash {:?} and node key {:?} mismatch.",
                    state_node_id,
                    node_key
                ))
            }
        } else {
            Err(format_err!(
                "State node is none by node key {:?}.",
                node_key
            ))
        }
    }

    pub async fn get_accumulator_node_by_node_hash(
        &self,
        node_key: HashValue,
        accumulator_type: AccumulatorStoreType,
    ) -> Result<(PeerId, AccumulatorNode)> {
        let peer_id = self.random_peer()?;
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
        max_size: usize,
    ) -> Result<Vec<HashValue>> {
        let peer_id = self.random_peer()?;
        let request = GetBlockIds {
            start_number,
            reverse,
            max_size,
        };
        self.client.get_block_ids(peer_id, request).await
    }

    pub async fn get_blocks(&self, ids: Vec<HashValue>) -> Result<Vec<Option<Block>>> {
        let peer_id = self.random_peer()?;
        let blocks: Vec<Option<Block>> =
            self.client.get_blocks(peer_id.clone(), ids.clone()).await?;
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
                        Some(block)
                    }
                } else {
                    None
                }
            })
            .collect())
    }
}
