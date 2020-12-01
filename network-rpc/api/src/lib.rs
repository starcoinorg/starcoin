// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

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
use starcoin_types::block::{Block, BlockHeader, BlockInfo, BlockNumber};
use starcoin_types::peer_info::PeerId;
use starcoin_types::transaction::{SignedUserTransaction, TransactionInfo};

mod remote_chain_state;

pub use network_rpc_core::RawRpcClient;
pub use remote_chain_state::RemoteChainStateReader;

use starcoin_types::account_address::AccountAddress;
use starcoin_types::account_state::AccountState;

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
    pub max_size: u64,
    pub step: u64,
    pub reverse: bool,
}

impl GetBlockHeaders {
    pub fn into_numbers(self, number: BlockNumber) -> Vec<BlockNumber> {
        let mut numbers = Vec::new();
        let mut last_number = number;
        loop {
            if numbers.len() as u64 >= self.max_size {
                break;
            }

            last_number = if self.reverse {
                if last_number < self.step {
                    break;
                }
                last_number - self.step
            } else {
                last_number + self.step
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

impl Into<starcoin_types::block::BlockBody> for BlockBody {
    fn into(self) -> starcoin_types::block::BlockBody {
        starcoin_types::block::BlockBody::new(self.transactions, self.uncles)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetBlockHeadersByNumber {
    pub number: BlockNumber,
    pub max_size: u64,
    pub step: u64,
}

impl Into<Vec<BlockNumber>> for GetBlockHeadersByNumber {
    fn into(self) -> Vec<BlockNumber> {
        let mut numbers = Vec::new();
        let mut last_number = self.number;
        loop {
            if numbers.len() as u64 >= self.max_size {
                break;
            }
            numbers.push(last_number);
            if last_number < self.step {
                break;
            }
            last_number -= self.step
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
    pub fn new(number: BlockNumber, step: u64, max_size: u64) -> Self {
        GetBlockHeadersByNumber {
            number,
            max_size,
            step,
        }
    }
}

impl GetBlockHeaders {
    pub fn new(block_id: HashValue, step: u64, reverse: bool, max_size: u64) -> Self {
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

/// Get block id list by block number, the `start_number` is include.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetBlockIds {
    pub start_number: BlockNumber,
    pub reverse: bool,
    pub max_size: u64,
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

    ///Get txns from txpool TODO rename
    fn get_txns_from_pool(
        &self,
        peer_id: PeerId,
        req: GetTxns,
    ) -> BoxFuture<Result<TransactionsData>>;

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

    fn get_headers(
        &self,
        peer_id: PeerId,
        request: GetBlockHeaders,
    ) -> BoxFuture<Result<Vec<BlockHeader>>>;

    fn get_block_infos(
        &self,
        peer_id: PeerId,
        hashes: Vec<HashValue>,
    ) -> BoxFuture<Result<Vec<BlockInfo>>>;

    fn get_bodies_by_hash(
        &self,
        peer_id: PeerId,
        hashes: Vec<HashValue>,
    ) -> BoxFuture<Result<Vec<BlockBody>>>;

    fn get_headers_by_hash(
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

    fn get_block_ids(&self, peer_id: PeerId, req: GetBlockIds)
        -> BoxFuture<Result<Vec<HashValue>>>;

    fn get_blocks(
        &self,
        peer_id: PeerId,
        ids: Vec<HashValue>,
    ) -> BoxFuture<Result<Vec<Option<Block>>>>;
}
