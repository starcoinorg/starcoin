// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use futures::future::BoxFuture;
use network_rpc_core::{NetRpcError, RpcErrorCode};
use network_rpc_derive::*;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use starcoin_accumulator::node::AccumulatorStoreType;
use starcoin_accumulator::AccumulatorNode;
use starcoin_crypto::HashValue;
use starcoin_state_api::StateWithProof;
use starcoin_state_tree::StateNode;
use starcoin_types::access_path::AccessPath;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::account_state::AccountState;
use starcoin_types::block::{Block, BlockHeader, BlockInfo, BlockNumber};
use starcoin_types::peer_info::{PeerId, RpcInfo};
use starcoin_types::transaction::{SignedUserTransaction, Transaction, TransactionInfo};

mod remote_chain_state;

pub use network_rpc_core::RawRpcClient;
pub use remote_chain_state::RemoteChainStateReader;

pub use starcoin_types::block::BlockBody;

pub const MAX_BLOCK_REQUEST_SIZE: u64 = 50;
pub const MAX_BLOCK_HEADER_REQUEST_SIZE: u64 = 1000;
pub const MAX_TXN_REQUEST_SIZE: u64 = 1000;
pub const MAX_BLOCK_INFO_REQUEST_SIZE: u64 = 1000;
pub const MAX_BLOCK_IDS_REQUEST_SIZE: u64 = 10000;

pub static RPC_INFO: Lazy<RpcInfo> = Lazy::new(|| RpcInfo::new(gen_client::get_rpc_info()));

pub trait RpcRequest {
    fn verify(&self) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetBlockHeadersByNumber {
    pub number: BlockNumber,
    pub max_size: u64,
    pub step: u64,
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

impl RpcRequest for GetBlockHeadersByNumber {
    fn verify(&self) -> Result<()> {
        if self.max_size > MAX_BLOCK_REQUEST_SIZE {
            return Err(NetRpcError::new(
                RpcErrorCode::BadRequest,
                format!("max_size is too big > {}", MAX_BLOCK_REQUEST_SIZE),
            )
            .into());
        }
        Ok(())
    }
}

#[allow(clippy::from_over_into)]
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

impl IntoIterator for GetBlockHeadersByNumber {
    type Item = BlockNumber;
    type IntoIter = std::vec::IntoIter<BlockNumber>;

    fn into_iter(self) -> Self::IntoIter {
        let vec: Vec<BlockNumber> = self.into();
        vec.into_iter()
    }
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct GetAccumulatorNodeByNodeHash {
    pub node_hash: HashValue,
    pub accumulator_storage_type: AccumulatorStoreType,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetTxnsWithSize {
    pub max_size: u64,
}

impl RpcRequest for GetTxnsWithSize {
    fn verify(&self) -> Result<()> {
        if self.max_size > MAX_TXN_REQUEST_SIZE {
            return Err(NetRpcError::new(
                RpcErrorCode::BadRequest,
                format!("max_size is too big > {}", MAX_TXN_REQUEST_SIZE),
            )
            .into());
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetTxnsWithHash {
    pub ids: Vec<HashValue>,
}

impl GetTxnsWithHash {
    pub fn len(&self) -> usize {
        self.ids.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl RpcRequest for GetTxnsWithHash {
    fn verify(&self) -> Result<()> {
        if self.ids.len() as u64 > MAX_TXN_REQUEST_SIZE {
            return Err(NetRpcError::new(
                RpcErrorCode::BadRequest,
                format!("max_size is too big > {}", MAX_TXN_REQUEST_SIZE),
            )
            .into());
        }
        Ok(())
    }
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

impl RpcRequest for GetBlockIds {
    fn verify(&self) -> Result<()> {
        if self.max_size as u64 > MAX_BLOCK_IDS_REQUEST_SIZE {
            return Err(NetRpcError::new(
                RpcErrorCode::BadRequest,
                format!("max_size is too big > {}", MAX_BLOCK_IDS_REQUEST_SIZE),
            )
            .into());
        }
        Ok(())
    }
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

    ///Get txns from txpool
    fn get_txns_from_pool(
        &self,
        peer_id: PeerId,
        req: GetTxnsWithSize,
    ) -> BoxFuture<Result<Vec<SignedUserTransaction>>>;

    ///Get txns with hash from txpool
    fn get_txns_with_hash_from_pool(
        &self,
        peer_id: PeerId,
        req: GetTxnsWithHash,
    ) -> BoxFuture<Result<Vec<Option<SignedUserTransaction>>>>;

    fn get_txns(
        &self,
        peer_id: PeerId,
        req: GetTxnsWithHash,
    ) -> BoxFuture<Result<Vec<Option<Transaction>>>>;

    fn get_txn_infos(
        &self,
        peer_id: PeerId,
        block_id: HashValue,
    ) -> BoxFuture<Result<Option<Vec<TransactionInfo>>>>;

    fn get_headers_by_number(
        &self,
        peer_id: PeerId,
        request: GetBlockHeadersByNumber,
    ) -> BoxFuture<Result<Vec<Option<BlockHeader>>>>;

    fn get_block_infos(
        &self,
        peer_id: PeerId,
        hashes: Vec<HashValue>,
    ) -> BoxFuture<Result<Vec<Option<BlockInfo>>>>;

    fn get_bodies_by_hash(
        &self,
        peer_id: PeerId,
        hashes: Vec<HashValue>,
    ) -> BoxFuture<Result<Vec<Option<BlockBody>>>>;

    fn get_headers_by_hash(
        &self,
        peer_id: PeerId,
        hashes: Vec<HashValue>,
    ) -> BoxFuture<Result<Vec<Option<BlockHeader>>>>;

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
