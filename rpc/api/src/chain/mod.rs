// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

pub type ChainClient = jsonrpsee::async_client::Client;
use crate::types::pubsub::EventFilter;
use crate::types::{
    BlockColorView, BlockHeaderView, BlockInfoView, BlockView, ChainId, ChainInfoView,
    MultiStateView, StrView, TransactionEventResponse, TransactionInfoView,
    TransactionInfoViewEnum, TransactionInfoWithProofView, TransactionView,
};
use crate::FutureResult;
use anyhow::Result;
use jsonrpsee::{
    core::RegisterMethodError,
    Methods, RpcModule,
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
use std::sync::Arc;
pub trait ChainApi {
    fn id(&self) -> Result<ChainId>;

    /// Get main chain info
    fn info(&self) -> FutureResult<ChainInfoView>;
    /// Get chain block info
    fn get_block_by_hash(
        &self,
        block_hash: HashValue,
        option: Option<GetBlockOption>,
    ) -> FutureResult<Option<BlockView>>;

    /// Get chain blocks by number
    fn get_block_by_number(
        &self,
        number: BlockNumber,
        option: Option<GetBlockOption>,
    ) -> FutureResult<Option<BlockView>>;
    /// Get latest `count` blocks before `number`. if `number` is absent, use head block number.
    fn get_blocks_by_number(
        &self,
        number: Option<BlockNumber>,
        count: u64,
        option: Option<GetBlocksOption>,
    ) -> FutureResult<Vec<BlockView>>;
    fn get_block_info_by_number(&self, number: BlockNumber) -> FutureResult<Option<BlockInfoView>>;
    fn get_block_info_by_hash(&self, id: HashValue) -> FutureResult<Option<BlockInfoView>>;
    fn get_block_info_by_number2(
        &self,
        number: BlockNumber,
    ) -> FutureResult<Option<BlockInfoView2>>;

    /// Get chain transactions
    fn get_transaction(
        &self,
        transaction_hash: HashValue,
        option: Option<GetTransactionOption>,
    ) -> FutureResult<Option<TransactionView>>;
    /// Get vm2 chain transactions
    fn get_transaction2(
        &self,
        transaction_hash: HashValue,
        option: Option<GetTransactionOption>,
    ) -> FutureResult<Option<TransactionView2>>;
    /// Get confirmed transaction info based on current main chain selection.
    fn get_transaction_info(
        &self,
        transaction_hash: HashValue,
    ) -> FutureResult<Option<TransactionInfoView>>;
    /// Get confirmed VM2 transaction info based on current main chain selection.
    fn get_transaction_info2(
        &self,
        transaction_hash: HashValue,
    ) -> FutureResult<Option<TransactionInfoView2>>;

    /// Get chain transactions infos by block id
    fn get_block_txn_infos(&self, block_hash: HashValue) -> FutureResult<Vec<TransactionInfoView>>;
    /// Get chain vm2 transactions infos by block id
    fn get_block_txn_infos2(
        &self,
        block_hash: HashValue,
    ) -> FutureResult<Vec<TransactionInfoView2>>;

    /// Get chain transactions infos by block id in sequence (both VM1 and VM2)
    fn get_block_txn_infos_in_seq(
        &self,
        block_hash: HashValue,
    ) -> FutureResult<Vec<TransactionInfoViewEnum>>;

    /// Get txn info of a txn at `idx` of block `block_id`
    fn get_txn_info_by_block_and_index(
        &self,
        block_hash: HashValue,
        idx: u64,
    ) -> FutureResult<Option<TransactionInfoView>>;
    /// Get txn info of a vm2 txn at `idx` of block `block_id`
    fn get_txn_info_by_block_and_index2(
        &self,
        block_hash: HashValue,
        idx: u64,
    ) -> FutureResult<Option<TransactionInfoView2>>;
    fn get_events_by_txn_hash(
        &self,
        txn_hash: HashValue,
        option: Option<GetEventOption>,
    ) -> FutureResult<Vec<TransactionEventResponse>>;
    fn get_events_by_txn_hash2(
        &self,
        txn_hash: HashValue,
        option: Option<GetEventOption>,
    ) -> FutureResult<Vec<TransactionEventResponse2>>;
    fn get_events(
        &self,
        filter: EventFilter,
        option: Option<GetEventOption>,
    ) -> FutureResult<Vec<TransactionEventResponse>>;

    /// Get headers by ids.
    fn get_headers(&self, ids: Vec<HashValue>) -> FutureResult<Vec<BlockHeaderView>>;

    /// Get transaction info list
    /// `start_global_index` is the transaction global index
    fn get_transaction_infos(
        &self,
        start_global_index: u64,
        reverse: bool,
        max_size: u64,
    ) -> FutureResult<Vec<TransactionInfoView>>;

    /// Get vm2 transaction info list
    /// `start_global_index` is the transaction global index
    fn get_transaction_infos2(
        &self,
        start_global_index: u64,
        reverse: bool,
        max_size: u64,
    ) -> FutureResult<Vec<TransactionInfoView2>>;
    fn get_transaction_proof(
        &self,
        block_hash: HashValue,
        transaction_global_index: u64,
        event_index: Option<u64>,
        access_path: Option<StrView<AccessPath>>,
    ) -> FutureResult<Option<TransactionInfoWithProofView>>;
    fn get_transaction_proof_raw(
        &self,
        block_hash: HashValue,
        transaction_global_index: u64,
        event_index: Option<u64>,
        access_path: Option<StrView<AccessPath>>,
    ) -> FutureResult<Option<StrView<Vec<u8>>>>;
    fn get_transaction_proof2(
        &self,
        block_hash: HashValue,
        transaction_global_index: u64,
        event_index: Option<u64>,
        access_path: Option<MultiAccessPath>,
    ) -> FutureResult<Option<TransactionInfoWithProofView>>;
    fn get_transaction_proof2_raw(
        &self,
        block_hash: HashValue,
        transaction_global_index: u64,
        event_index: Option<u64>,
        access_path: Option<MultiAccessPath>,
    ) -> FutureResult<Option<StrView2<Vec<u8>>>>;
    fn get_vm_multi_state(&self, block_hash: HashValue) -> FutureResult<Option<MultiStateView>>;

    /// Get block ghostdag data
    fn get_ghostdagdata(&self, ids: Vec<HashValue>) -> FutureResult<Vec<Option<GhostdagData>>>;

    /// Get block color based on the current main chain selection.
    fn get_current_block_color(
        &self,
        block_hash: HashValue,
    ) -> FutureResult<Option<BlockColorView>>;
}

/// Build jsonrpsee methods from legacy `ChainApi`.
///
/// This keeps the existing `ChainApi` trait unchanged and enables incremental
/// server runtime migration to jsonrpsee.
pub fn chain_methods<T>(api: T) -> std::result::Result<Methods, RegisterMethodError>
where
    T: ChainApi + Send + Sync + 'static,
{
    let mut module = RpcModule::new(Arc::new(api));

    module.register_method("chain.id", |_, api, _| api.id().map_err(crate::map_jsonrpc_err))?;

    module.register_async_method("chain.info", |_, api, _| async move {
        api.info().await.map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("chain.get_block_by_hash", |params, api, _| async move {
        let (block_hash, option): (HashValue, Option<GetBlockOption>) = params.parse()?;
        api.get_block_by_hash(block_hash, option)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("chain.get_block_by_number", |params, api, _| async move {
        let (number, option): (BlockNumber, Option<GetBlockOption>) = params.parse()?;
        api.get_block_by_number(number, option)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("chain.get_blocks_by_number", |params, api, _| async move {
        let (number, count, option): (Option<BlockNumber>, u64, Option<GetBlocksOption>) =
            params.parse()?;
        api.get_blocks_by_number(number, count, option)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("chain.get_block_info_by_number", |params, api, _| async move {
        let number: BlockNumber = params.one()?;
        api.get_block_info_by_number(number)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("chain.get_block_info_by_hash", |params, api, _| async move {
        let id: HashValue = params.one()?;
        api.get_block_info_by_hash(id).await.map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("chain.get_block_info_by_number2", |params, api, _| async move {
        let number: BlockNumber = params.one()?;
        api.get_block_info_by_number2(number)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("chain.get_transaction", |params, api, _| async move {
        let (transaction_hash, option): (HashValue, Option<GetTransactionOption>) = params.parse()?;
        api.get_transaction(transaction_hash, option)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("chain.get_transaction2", |params, api, _| async move {
        let (transaction_hash, option): (HashValue, Option<GetTransactionOption>) = params.parse()?;
        api.get_transaction2(transaction_hash, option)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("chain.get_transaction_info", |params, api, _| async move {
        let transaction_hash: HashValue = params.one()?;
        api.get_transaction_info(transaction_hash)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("chain.get_transaction_info2", |params, api, _| async move {
        let transaction_hash: HashValue = params.one()?;
        api.get_transaction_info2(transaction_hash)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("chain.get_block_txn_infos", |params, api, _| async move {
        let block_hash: HashValue = params.one()?;
        api.get_block_txn_infos(block_hash)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("chain.get_block_txn_infos2", |params, api, _| async move {
        let block_hash: HashValue = params.one()?;
        api.get_block_txn_infos2(block_hash)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("chain.get_block_txn_infos_in_seq", |params, api, _| async move {
        let block_hash: HashValue = params.one()?;
        api.get_block_txn_infos_in_seq(block_hash)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("chain.get_txn_info_by_block_and_index", |params, api, _| async move {
        let (block_hash, idx): (HashValue, u64) = params.parse()?;
        api.get_txn_info_by_block_and_index(block_hash, idx)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("chain.get_txn_info_by_block_and_index2", |params, api, _| async move {
        let (block_hash, idx): (HashValue, u64) = params.parse()?;
        api.get_txn_info_by_block_and_index2(block_hash, idx)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("chain.get_events_by_txn_hash", |params, api, _| async move {
        let (txn_hash, option): (HashValue, Option<GetEventOption>) = params.parse()?;
        api.get_events_by_txn_hash(txn_hash, option)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("chain.get_events_by_txn_hash2", |params, api, _| async move {
        let (txn_hash, option): (HashValue, Option<GetEventOption>) = params.parse()?;
        api.get_events_by_txn_hash2(txn_hash, option)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("chain.get_events", |params, api, _| async move {
        let (filter, option): (EventFilter, Option<GetEventOption>) = params.parse()?;
        api.get_events(filter, option)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("chain.get_headers", |params, api, _| async move {
        let ids: Vec<HashValue> = params.one()?;
        api.get_headers(ids).await.map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("chain.get_transaction_infos", |params, api, _| async move {
        let (start_global_index, reverse, max_size): (u64, bool, u64) = params.parse()?;
        api.get_transaction_infos(start_global_index, reverse, max_size)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("chain.get_transaction_infos2", |params, api, _| async move {
        let (start_global_index, reverse, max_size): (u64, bool, u64) = params.parse()?;
        api.get_transaction_infos2(start_global_index, reverse, max_size)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("chain.get_transaction_proof", |params, api, _| async move {
        let (block_hash, transaction_global_index, event_index, access_path): (
            HashValue,
            u64,
            Option<u64>,
            Option<StrView<AccessPath>>,
        ) = params.parse()?;
        api.get_transaction_proof(block_hash, transaction_global_index, event_index, access_path)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("chain.get_transaction_proof_raw", |params, api, _| async move {
        let (block_hash, transaction_global_index, event_index, access_path): (
            HashValue,
            u64,
            Option<u64>,
            Option<StrView<AccessPath>>,
        ) = params.parse()?;
        api.get_transaction_proof_raw(block_hash, transaction_global_index, event_index, access_path)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("chain.get_transaction_proof2", |params, api, _| async move {
        let (block_hash, transaction_global_index, event_index, access_path): (
            HashValue,
            u64,
            Option<u64>,
            Option<MultiAccessPath>,
        ) = params.parse()?;
        api.get_transaction_proof2(block_hash, transaction_global_index, event_index, access_path)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("chain.get_transaction_proof2_raw", |params, api, _| async move {
        let (block_hash, transaction_global_index, event_index, access_path): (
            HashValue,
            u64,
            Option<u64>,
            Option<MultiAccessPath>,
        ) = params.parse()?;
        api.get_transaction_proof2_raw(block_hash, transaction_global_index, event_index, access_path)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("chain.get_vm_multi_state", |params, api, _| async move {
        let block_hash: HashValue = params.one()?;
        api.get_vm_multi_state(block_hash)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("chain.get_ghostdagdata", |params, api, _| async move {
        let ids: Vec<HashValue> = params.one()?;
        api.get_ghostdagdata(ids)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("chain.get_current_block_color", |params, api, _| async move {
        let block_hash: HashValue = params.one()?;
        api.get_current_block_color(block_hash)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;

    Ok(module.into())
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

