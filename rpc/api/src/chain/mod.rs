// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use crate::types::pubsub::EventFilter;
use crate::types::{
    BlockColorView, BlockHeaderView, BlockInfoView, BlockView, ChainId, ChainInfoView,
    MultiStateView, StrView, TransactionEventResponse, TransactionInfoView,
    TransactionInfoViewEnum, TransactionInfoWithProofView, TransactionView,
};
use jsonrpsee::{
    core::{RegisterMethodError, RpcResult},
    proc_macros::rpc,
    Methods,
};
use schemars::{self, JsonSchema};
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;
use starcoin_dag::types::ghostdata::GhostdagData;
use starcoin_types::block::BlockNumber;
use starcoin_types::multi_access_path::MultiAccessPath;
use starcoin_vm2_rpc_api::{block_info_view2::BlockInfoView2, transaction_view2::TransactionView2};
use starcoin_vm2_types::view::{
    StrView as StrView2, TransactionEventResponse as TransactionEventResponse2,
    TransactionInfoView as TransactionInfoView2,
};
use starcoin_vm_types::access_path::AccessPath;

use starcoin_rpc_schema_derive::rpc_schema;

#[rpc_schema]
#[rpc(client, server, namespace = "chain", namespace_separator = ".")]
pub trait ChainApi {
    #[method(name = "id")]
    async fn id(&self) -> RpcResult<ChainId>;

    /// Get main chain info
    #[method(name = "info")]
    async fn info(&self) -> RpcResult<ChainInfoView>;

    /// Get chain block info
    #[method(name = "get_block_by_hash")]
    async fn get_block_by_hash(
        &self,
        block_hash: HashValue,
        option: Option<GetBlockOption>,
    ) -> RpcResult<Option<BlockView>>;

    /// Get chain blocks by number
    #[method(name = "get_block_by_number")]
    async fn get_block_by_number(
        &self,
        number: BlockNumber,
        option: Option<GetBlockOption>,
    ) -> RpcResult<Option<BlockView>>;

    /// Get latest `count` blocks before `number`. if `number` is absent, use head block number.
    #[method(name = "get_blocks_by_number")]
    async fn get_blocks_by_number(
        &self,
        number: Option<BlockNumber>,
        count: u64,
        option: Option<GetBlocksOption>,
    ) -> RpcResult<Vec<BlockView>>;

    #[method(name = "get_block_info_by_number")]
    async fn get_block_info_by_number(
        &self,
        number: BlockNumber,
    ) -> RpcResult<Option<BlockInfoView>>;

    #[method(name = "get_block_info_by_hash")]
    async fn get_block_info_by_hash(&self, id: HashValue) -> RpcResult<Option<BlockInfoView>>;

    #[method(name = "get_block_info_by_number2")]
    async fn get_block_info_by_number2(
        &self,
        number: BlockNumber,
    ) -> RpcResult<Option<BlockInfoView2>>;

    /// Get chain transactions
    #[method(name = "get_transaction")]
    async fn get_transaction(
        &self,
        transaction_hash: HashValue,
        option: Option<GetTransactionOption>,
    ) -> RpcResult<Option<TransactionView>>;

    /// Get vm2 chain transactions
    #[method(name = "get_transaction2")]
    async fn get_transaction2(
        &self,
        transaction_hash: HashValue,
        option: Option<GetTransactionOption>,
    ) -> RpcResult<Option<TransactionView2>>;

    /// Get confirmed transaction info based on current main chain selection.
    #[method(name = "get_transaction_info")]
    async fn get_transaction_info(
        &self,
        transaction_hash: HashValue,
    ) -> RpcResult<Option<TransactionInfoView>>;

    /// Get confirmed VM2 transaction info based on current main chain selection.
    #[method(name = "get_transaction_info2")]
    async fn get_transaction_info2(
        &self,
        transaction_hash: HashValue,
    ) -> RpcResult<Option<TransactionInfoView2>>;

    /// Get chain transactions infos by block id
    #[method(name = "get_block_txn_infos")]
    async fn get_block_txn_infos(
        &self,
        block_hash: HashValue,
    ) -> RpcResult<Vec<TransactionInfoView>>;

    /// Get chain vm2 transactions infos by block id
    #[method(name = "get_block_txn_infos2")]
    async fn get_block_txn_infos2(
        &self,
        block_hash: HashValue,
    ) -> RpcResult<Vec<TransactionInfoView2>>;

    /// Get chain transactions infos by block id in sequence (both VM1 and VM2)
    #[method(name = "get_block_txn_infos_in_seq")]
    async fn get_block_txn_infos_in_seq(
        &self,
        block_hash: HashValue,
    ) -> RpcResult<Vec<TransactionInfoViewEnum>>;

    /// Get txn info of a txn at `idx` of block `block_id`
    #[method(name = "get_txn_info_by_block_and_index")]
    async fn get_txn_info_by_block_and_index(
        &self,
        block_hash: HashValue,
        idx: u64,
    ) -> RpcResult<Option<TransactionInfoView>>;

    /// Get txn info of a vm2 txn at `idx` of block `block_id`
    #[method(name = "get_txn_info_by_block_and_index2")]
    async fn get_txn_info_by_block_and_index2(
        &self,
        block_hash: HashValue,
        idx: u64,
    ) -> RpcResult<Option<TransactionInfoView2>>;

    #[method(name = "get_events_by_txn_hash")]
    async fn get_events_by_txn_hash(
        &self,
        txn_hash: HashValue,
        option: Option<GetEventOption>,
    ) -> RpcResult<Vec<TransactionEventResponse>>;

    #[method(name = "get_events_by_txn_hash2")]
    async fn get_events_by_txn_hash2(
        &self,
        txn_hash: HashValue,
        option: Option<GetEventOption>,
    ) -> RpcResult<Vec<TransactionEventResponse2>>;

    #[method(name = "get_events")]
    async fn get_events(
        &self,
        filter: EventFilter,
        option: Option<GetEventOption>,
    ) -> RpcResult<Vec<TransactionEventResponse>>;

    /// Get headers by ids.
    #[method(name = "get_headers")]
    async fn get_headers(&self, ids: Vec<HashValue>) -> RpcResult<Vec<BlockHeaderView>>;

    /// Get transaction info list
    /// `start_global_index` is the transaction global index
    #[method(name = "get_transaction_infos")]
    async fn get_transaction_infos(
        &self,
        start_global_index: u64,
        reverse: bool,
        max_size: u64,
    ) -> RpcResult<Vec<TransactionInfoView>>;

    /// Get vm2 transaction info list
    /// `start_global_index` is the transaction global index
    #[method(name = "get_transaction_infos2")]
    async fn get_transaction_infos2(
        &self,
        start_global_index: u64,
        reverse: bool,
        max_size: u64,
    ) -> RpcResult<Vec<TransactionInfoView2>>;

    #[method(name = "get_transaction_proof")]
    async fn get_transaction_proof(
        &self,
        block_hash: HashValue,
        transaction_global_index: u64,
        event_index: Option<u64>,
        access_path: Option<StrView<AccessPath>>,
    ) -> RpcResult<Option<TransactionInfoWithProofView>>;

    #[method(name = "get_transaction_proof_raw")]
    async fn get_transaction_proof_raw(
        &self,
        block_hash: HashValue,
        transaction_global_index: u64,
        event_index: Option<u64>,
        access_path: Option<StrView<AccessPath>>,
    ) -> RpcResult<Option<StrView<Vec<u8>>>>;

    #[method(name = "get_transaction_proof2")]
    async fn get_transaction_proof2(
        &self,
        block_hash: HashValue,
        transaction_global_index: u64,
        event_index: Option<u64>,
        access_path: Option<MultiAccessPath>,
    ) -> RpcResult<Option<TransactionInfoWithProofView>>;

    #[method(name = "get_transaction_proof2_raw")]
    async fn get_transaction_proof2_raw(
        &self,
        block_hash: HashValue,
        transaction_global_index: u64,
        event_index: Option<u64>,
        access_path: Option<MultiAccessPath>,
    ) -> RpcResult<Option<StrView2<Vec<u8>>>>;

    #[method(name = "get_vm_multi_state")]
    async fn get_vm_multi_state(&self, block_hash: HashValue) -> RpcResult<Option<MultiStateView>>;

    /// Get block ghostdag data
    #[method(name = "get_ghostdagdata")]
    async fn get_ghostdagdata(&self, ids: Vec<HashValue>) -> RpcResult<Vec<Option<GhostdagData>>>;

    /// Get block color based on the current main chain selection.
    #[method(name = "get_current_block_color")]
    async fn get_current_block_color(
        &self,
        block_hash: HashValue,
    ) -> RpcResult<Option<BlockColorView>>;
}

pub use ChainApiClient as ChainApiRpcClient;
pub use ChainApiServer as ChainApiRpcServer;

/// Build jsonrpsee methods from legacy `ChainApi`.
pub fn chain_methods<T>(api: T) -> std::result::Result<Methods, RegisterMethodError>
where
    T: ChainApiServer + Send + Sync + 'static,
{
    Ok(ChainApiServer::into_rpc(api).into())
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

    #[serde(default)]
    pub raw: bool,
}

#[derive(Copy, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct GetBlocksOption {
    #[serde(default = "defautl_true")]
    pub reverse: bool,
}

fn defautl_true() -> bool {
    true
}

#[derive(Copy, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct GetEventOption {
    #[serde(default)]
    pub decode: bool,
}
