// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use anyhow::Result;
use starcoin_crypto::HashValue;
use starcoin_state_api::{ChainState, ChainStateReader};
use starcoin_types::stress_test::TPS;
use starcoin_types::{
    block::{Block, BlockHeader, BlockInfo, BlockNumber},
    transaction::{Transaction, TransactionInfo},
    U256,
};
use starcoin_vm_types::on_chain_resource::{EpochInfo, GlobalTimeOnChain};

pub trait ChainReader {
    fn head_block(&self) -> Block;
    fn current_header(&self) -> BlockHeader;
    fn get_header(&self, hash: HashValue) -> Result<Option<BlockHeader>>;
    fn get_header_by_number(&self, number: BlockNumber) -> Result<Option<BlockHeader>>;
    fn get_block_by_number(&self, number: BlockNumber) -> Result<Option<Block>>;
    /// Get latest `count` blocks before `number`. if `number` is absent, use head block number.
    fn get_blocks_by_number(&self, number: Option<BlockNumber>, count: u64) -> Result<Vec<Block>>;
    fn get_block(&self, hash: HashValue) -> Result<Option<Block>>;
    fn get_transaction(&self, hash: HashValue) -> Result<Option<Transaction>>;
    /// Get transaction info by transaction's hash
    fn get_transaction_info(&self, txn_hash: HashValue) -> Result<Option<TransactionInfo>>;
    fn get_latest_block_by_uncle(&self, uncle_id: HashValue, times: u64) -> Result<Option<Block>>;

    /// get txn info at version in main chain.
    fn get_transaction_info_by_version(&self, version: u64) -> Result<Option<TransactionInfo>>;

    fn chain_state_reader(&self) -> &dyn ChainStateReader;
    fn get_block_info(&self, block_id: Option<HashValue>) -> Result<Option<BlockInfo>>;
    fn get_total_difficulty(&self) -> Result<U256>;
    fn exist_block(&self, block_id: HashValue) -> bool;
    fn epoch_info(&self) -> Result<EpochInfo>;
    fn get_epoch_info_by_number(&self, number: Option<BlockNumber>) -> Result<EpochInfo>;
    fn get_global_time_by_number(&self, number: BlockNumber) -> Result<GlobalTimeOnChain>;
    /// Get block id vec by BlockNumber, `start_number`'s block id is include.
    fn get_block_ids(
        &self,
        start_number: BlockNumber,
        reverse: bool,
        max_size: usize,
    ) -> Result<Vec<HashValue>>;
    fn get_block_info_by_number(&self, number: BlockNumber) -> Result<Option<BlockInfo>>;
    fn total_txns_in_blocks(
        &self,
        start_number: BlockNumber,
        end_number: BlockNumber,
    ) -> Result<u64>;
    /// Get tps for an epoch. The epoch includes the block given by `number`. If `number` is absent, return tps for the latest epoch
    fn tps(&self, number: Option<BlockNumber>) -> Result<TPS>;
}

pub trait ChainWriter {
    /// execute and insert block to current chain.
    fn apply(&mut self, block: Block) -> Result<()>;

    /// insert block to current chain.
    fn apply_without_execute(
        &mut self,
        block: Block,
        remote_state_chain: &dyn ChainStateReader,
    ) -> Result<()>;

    fn chain_state(&mut self) -> &dyn ChainState;
}

/// `Chain` is a trait that defines a single Chain.
pub trait Chain: ChainReader + ChainWriter {}
