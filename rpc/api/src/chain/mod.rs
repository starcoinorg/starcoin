// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

pub use self::gen_client::Client as ChainClient;
use crate::types::pubsub::EventFilter;
use crate::types::{
    BlockHeaderView, BlockSummaryView, BlockView, ChainId, ChainInfoView, EpochUncleSummaryView,
    TransactionEventView, TransactionInfoView, TransactionView,
};
use crate::FutureResult;
use jsonrpc_core::Result;
use jsonrpc_derive::rpc;
use starcoin_crypto::HashValue;
use starcoin_types::block::BlockNumber;
use starcoin_types::stress_test::TPS;
use starcoin_vm_types::on_chain_resource::{EpochInfo, GlobalTimeOnChain};

#[rpc]
pub trait ChainApi {
    #[rpc(name = "chain.id")]
    fn id(&self) -> Result<ChainId>;

    /// Get main chain info
    #[rpc(name = "chain.info")]
    fn info(&self) -> FutureResult<ChainInfoView>;
    /// Get chain block info
    #[rpc(name = "chain.get_block_by_hash")]
    fn get_block_by_hash(&self, block_hash: HashValue) -> FutureResult<BlockView>;
    /// Get chain blocks by number
    #[rpc(name = "chain.get_block_by_number")]
    fn get_block_by_number(&self, number: BlockNumber) -> FutureResult<BlockView>;
    /// Get latest `count` blocks before `number`. if `number` is absent, use head block number.
    #[rpc(name = "chain.get_blocks_by_number")]
    fn get_blocks_by_number(
        &self,
        number: Option<BlockNumber>,
        count: u64,
    ) -> FutureResult<Vec<BlockView>>;
    /// Get chain transactions
    #[rpc(name = "chain.get_transaction")]
    fn get_transaction(&self, transaction_hash: HashValue) -> FutureResult<TransactionView>;
    /// Get chain transactions
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
    ) -> FutureResult<Vec<TransactionEventView>>;

    #[rpc(name = "chain.get_events")]
    fn get_events(&self, filter: EventFilter) -> FutureResult<Vec<TransactionEventView>>;

    /// Get current epoch info.
    #[rpc(name = "chain.epoch")]
    fn current_epoch(&self) -> FutureResult<EpochInfo>;

    /// Get epoch info by number.
    #[rpc(name = "chain.get_epoch_info_by_number")]
    fn get_epoch_info_by_number(&self, number: BlockNumber) -> FutureResult<EpochInfo>;

    /// Get global time by number.
    #[rpc(name = "chain.get_global_time_by_number")]
    fn get_global_time_by_number(&self, number: BlockNumber) -> FutureResult<GlobalTimeOnChain>;

    /// Get chain blocks by number
    #[rpc(name = "chain.get_block_by_uncle")]
    fn get_block_by_uncle(&self, uncle_hash: HashValue) -> FutureResult<Option<BlockView>>;

    /// Get tps by block number.
    #[rpc(name = "chain.tps")]
    fn tps(&self, number: Option<BlockNumber>) -> FutureResult<TPS>;

    /// Get uncles by number.
    #[rpc(name = "chain.get_epoch_uncles_by_number")]
    fn get_epoch_uncles_by_number(
        &self,
        number: BlockNumber,
    ) -> FutureResult<Vec<BlockSummaryView>>;

    /// Get headers by ids.
    #[rpc(name = "chain.get_headers")]
    fn get_headers(&self, ids: Vec<HashValue>) -> FutureResult<Vec<BlockHeaderView>>;

    /// Uncle path.
    #[rpc(name = "chain.uncle_path")]
    fn uncle_path(
        &self,
        block_id: HashValue,
        uncle_id: HashValue,
    ) -> FutureResult<Vec<BlockHeaderView>>;

    /// Epoch uncle summary by number.
    #[rpc(name = "chain.epoch_uncle_summary_by_number")]
    fn epoch_uncle_summary_by_number(
        &self,
        number: BlockNumber,
    ) -> FutureResult<EpochUncleSummaryView>;
}
