// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

pub use self::gen_client::Client as ChainClient;
use crate::types::pubsub::EventFilter;
use crate::types::{
    BlockHeaderView, BlockInfoView, BlockView, ChainId, ChainInfoView, StrView,
    TransactionEventResponse, TransactionInfoView, TransactionInfoWithProofView, TransactionView,
};
use crate::FutureResult;
use jsonrpc_core::Result;
use jsonrpc_derive::rpc;
use schemars::{self, JsonSchema};
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;
use starcoin_types::block::BlockNumber;
use starcoin_vm_types::access_path::AccessPath;

#[rpc(client, server, schema)]
pub trait ChainApi {
    #[rpc(name = "chain.id")]
    fn id(&self) -> Result<ChainId>;

    /// Get main chain info
    #[rpc(name = "chain.info")]
    fn info(&self) -> FutureResult<ChainInfoView>;
    /// Get chain block info
    #[rpc(name = "chain.get_block_by_hash")]
    fn get_block_by_hash(
        &self,
        block_hash: HashValue,
        option: Option<GetBlockOption>,
    ) -> FutureResult<Option<BlockView>>;

    /// Get chain blocks by number
    #[rpc(name = "chain.get_block_by_number")]
    fn get_block_by_number(
        &self,
        number: BlockNumber,
        option: Option<GetBlockOption>,
    ) -> FutureResult<Option<BlockView>>;
    /// Get latest `count` blocks before `number`. if `number` is absent, use head block number.
    #[rpc(name = "chain.get_blocks_by_number")]
    fn get_blocks_by_number(
        &self,
        number: Option<BlockNumber>,
        count: u64,
    ) -> FutureResult<Vec<BlockView>>;
    #[rpc(name = "chain.get_block_info_by_number")]
    fn get_block_info_by_number(&self, number: BlockNumber) -> FutureResult<Option<BlockInfoView>>;
    /// Get chain transactions
    #[rpc(name = "chain.get_transaction")]
    fn get_transaction(
        &self,
        transaction_hash: HashValue,
        option: Option<GetTransactionOption>,
    ) -> FutureResult<Option<TransactionView>>;
    /// Get chain transaction info
    #[rpc(name = "chain.get_transaction_info")]
    fn get_transaction_info(
        &self,
        transaction_hash: HashValue,
    ) -> FutureResult<Option<TransactionInfoView>>;

    /// Get chain transactions infos by block id
    #[rpc(name = "chain.get_block_txn_infos")]
    fn get_block_txn_infos(&self, block_hash: HashValue) -> FutureResult<Vec<TransactionInfoView>>;
    /// Get txn info of a txn at `idx` of block `block_id`
    #[rpc(name = "chain.get_txn_info_by_block_and_index")]
    fn get_txn_info_by_block_and_index(
        &self,
        block_hash: HashValue,
        idx: u64,
    ) -> FutureResult<Option<TransactionInfoView>>;

    #[rpc(name = "chain.get_events_by_txn_hash")]
    fn get_events_by_txn_hash(
        &self,
        txn_hash: HashValue,
        option: Option<GetEventOption>,
    ) -> FutureResult<Vec<TransactionEventResponse>>;
    #[rpc(name = "chain.get_events")]
    fn get_events(
        &self,
        filter: EventFilter,
        option: Option<GetEventOption>,
    ) -> FutureResult<Vec<TransactionEventResponse>>;

    /// Get headers by ids.
    #[rpc(name = "chain.get_headers")]
    fn get_headers(&self, ids: Vec<HashValue>) -> FutureResult<Vec<BlockHeaderView>>;

    /// Get transaction info list
    /// `start_global_index` is the transaction global index
    #[rpc(name = "chain.get_transaction_infos")]
    fn get_transaction_infos(
        &self,
        start_global_index: u64,
        reverse: bool,
        max_size: u64,
    ) -> FutureResult<Vec<TransactionInfoView>>;

    /// Get TransactionInfoWithProof, if the block with `block_hash` or transaction with `transaction_global_index` do not exists, return None.
    /// if `event_index` is some, also return the EventWithProof in current transaction event_root
    /// if `access_path` is some, also return the StateWithProof in current transaction state_root
    #[rpc(name = "chain.get_transaction_proof")]
    fn get_transaction_proof(
        &self,
        block_hash: HashValue,
        transaction_global_index: u64,
        event_index: Option<u64>,
        access_path: Option<StrView<AccessPath>>,
    ) -> FutureResult<Option<TransactionInfoWithProofView>>;
}

#[derive(Copy, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct GetTransactionOption {
    #[serde(default)]
    pub decode: bool,
}

#[derive(Copy, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct GetBlockOption {
    #[serde(default)]
    pub decode: bool,
}

#[derive(Copy, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct GetEventOption {
    #[serde(default)]
    pub decode: bool,
}

#[test]
fn test() {
    let schema = rpc_impl_ChainApi::gen_client::Client::gen_schema();
    let j = serde_json::to_string_pretty(&schema).unwrap();
    println!("{}", j);
}
