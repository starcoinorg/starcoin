// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use actix::Message;
use anyhow::Result;
use futures::future::BoxFuture;
use network_rpc_derive::*;
use serde::{Deserialize, Serialize};
use starcoin_accumulator::node::AccumulatorStoreType;
use starcoin_accumulator::AccumulatorNode;
use starcoin_crypto::HashValue;
use starcoin_state_api::StateWithProof;
use starcoin_state_tree::StateNode;
use starcoin_types::access_path::AccessPath;
use starcoin_types::block::{BlockHeader, BlockInfo, BlockNumber};
use starcoin_types::peer_info::PeerId;
use starcoin_types::transaction::{SignedUserTransaction, TransactionInfo};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TransactionsData {
    pub txns: Vec<SignedUserTransaction>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetBlockHeaders {
    pub block_id: HashValue,
    pub max_size: usize,
    pub step: usize,
    pub reverse: bool,
}

#[derive(Eq, Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct BlockBody {
    pub hash: HashValue,
    pub transactions: Vec<SignedUserTransaction>,
    pub uncles: Option<Vec<BlockHeader>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetBlockHeadersByNumber {
    pub number: BlockNumber,
    pub max_size: usize,
    pub step: usize,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct GetAccumulatorNodeByNodeHash {
    pub node_hash: HashValue,
    pub accumulator_storage_type: AccumulatorStoreType,
}

impl GetBlockHeadersByNumber {
    pub fn new(number: BlockNumber, step: usize, max_size: usize) -> Self {
        GetBlockHeadersByNumber {
            number,
            max_size,
            step,
        }
    }
}

impl GetBlockHeaders {
    pub fn new(block_id: HashValue, step: usize, reverse: bool, max_size: usize) -> Self {
        GetBlockHeaders {
            block_id,
            max_size,
            step,
            reverse,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetTxns {
    pub ids: Option<Vec<HashValue>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetStateWithProof {
    pub state_root: HashValue,
    pub access_path: AccessPath,
}

impl Message for GetStateWithProof {
    type Result = Result<StateWithProof>;
}

pub(crate) const DELAY_TIME: u64 = 15;

#[net_rpc]
pub trait NetworkRpc: Sized + Send + Sync + 'static {
    fn get_txns(&self, peer_id: PeerId, req: GetTxns) -> BoxFuture<Result<TransactionsData>>;

    fn get_txn_infos(
        &self,
        peer_id: PeerId,
        block_id: HashValue,
    ) -> BoxFuture<Result<Option<Vec<TransactionInfo>>>>;

    fn get_headers_by_number(
        &self,
        peer_id: PeerId,
        request: GetBlockHeadersByNumber,
    ) -> BoxFuture<Result<Vec<BlockHeader>>>;

    fn get_headers_with_peer(
        &self,
        peer_id: PeerId,
        request: GetBlockHeaders,
    ) -> BoxFuture<Result<Vec<BlockHeader>>>;

    fn get_info_by_hash(
        &self,
        peer_id: PeerId,
        hashes: Vec<HashValue>,
    ) -> BoxFuture<Result<Vec<BlockInfo>>>;

    fn get_body_by_hash(
        &self,
        peer_id: PeerId,
        hashs: Vec<HashValue>,
    ) -> BoxFuture<Result<Vec<BlockBody>>>;

    fn get_state_node_by_node_hash(
        &self,
        peer_id: PeerId,
        node_key: HashValue,
    ) -> BoxFuture<Result<StateNode>>;

    fn get_accumulator_node_by_node_hash(
        &self,
        peer_id: PeerId,
        request: GetAccumulatorNodeByNodeHash,
    ) -> BoxFuture<Result<AccumulatorNode>>;

    fn get_state_with_proof(
        &self,
        peer_id: PeerId,
        req: GetStateWithProof,
    ) -> BoxFuture<Result<StateWithProof>>;
}
