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

mod remote_chain_state;

pub use network_rpc_core::RawRpcClient;
pub use remote_chain_state::RemoteChainStateReader;

use starcoin_types::account_address::AccountAddress;
use starcoin_types::account_state::AccountState;

//TODO move this constants from types
pub use starcoin_types::CHAIN_PROTOCOL_NAME;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TransactionsData {
    pub txns: Vec<SignedUserTransaction>,
}

impl TransactionsData {
    pub fn get_txns(&self) -> &[SignedUserTransaction] {
        self.txns.as_slice()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetBlockHeaders {
    pub block_id: HashValue,
    pub max_size: usize,
    pub step: usize,
    pub reverse: bool,
}

impl GetBlockHeaders {
    pub fn into_numbers(self, number: BlockNumber) -> Vec<BlockNumber> {
        let mut numbers = Vec::new();
        let mut last_number = number;
        loop {
            if numbers.len() >= self.max_size {
                break;
            }

            last_number = if self.reverse {
                if last_number < self.step as u64 {
                    break;
                }
                last_number - self.step as u64
            } else {
                last_number + self.step as u64
            };
            numbers.push(last_number);
        }
        numbers
    }
}

#[derive(Eq, Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct BlockBody {
    pub hash: HashValue,
    pub transactions: Vec<SignedUserTransaction>,
    pub uncles: Option<Vec<BlockHeader>>,
}

impl BlockBody {
    pub fn id(&self) -> HashValue {
        self.hash
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetBlockHeadersByNumber {
    pub number: BlockNumber,
    pub max_size: usize,
    pub step: usize,
}

impl Into<Vec<BlockNumber>> for GetBlockHeadersByNumber {
    fn into(self) -> Vec<BlockNumber> {
        let mut numbers = Vec::new();
        let mut last_number = self.number;
        loop {
            if numbers.len() >= self.max_size {
                break;
            }
            numbers.push(last_number);
            if last_number < self.step as u64 {
                break;
            }
            last_number -= self.step as u64
        }
        numbers
    }
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetAccountState {
    pub state_root: HashValue,
    pub account_address: AccountAddress,
}

impl Message for GetStateWithProof {
    type Result = Result<StateWithProof>;
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct Ping {
    ///ping message, return same msg.
    pub msg: String,
    ///return by error
    pub err: bool,
}

#[net_rpc(client, server)]
pub trait NetworkRpc: Sized + Send + Sync + 'static {
    fn ping(&self, peer_id: PeerId, req: Ping) -> BoxFuture<Result<String>>;

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
        hashes: Vec<HashValue>,
    ) -> BoxFuture<Result<Vec<BlockBody>>>;

    fn get_header_by_hash(
        &self,
        peer_id: PeerId,
        hashes: Vec<HashValue>,
    ) -> BoxFuture<Result<Vec<BlockHeader>>>;

    fn get_state_node_by_node_hash(
        &self,
        peer_id: PeerId,
        node_key: HashValue,
    ) -> BoxFuture<Result<Option<StateNode>>>;

    fn get_accumulator_node_by_node_hash(
        &self,
        peer_id: PeerId,
        request: GetAccumulatorNodeByNodeHash,
    ) -> BoxFuture<Result<Option<AccumulatorNode>>>;

    fn get_state_with_proof(
        &self,
        peer_id: PeerId,
        req: GetStateWithProof,
    ) -> BoxFuture<Result<StateWithProof>>;

    fn get_account_state(
        &self,
        peer_id: PeerId,
        req: GetAccountState,
    ) -> BoxFuture<Result<Option<AccountState>>>;
}
