// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use anyhow::Result;
use starcoin_crypto::HashValue;
use starcoin_state_api::{ChainState, ChainStateReader};
use starcoin_types::block::BlockIdAndNumber;
use starcoin_types::startup_info::{ChainInfo, ChainStatus};
use starcoin_types::transaction::RichTransactionInfo;
use starcoin_types::{
    block::{Block, BlockHeader, BlockInfo, BlockNumber},
    transaction::Transaction,
    U256,
};
use starcoin_vm_types::on_chain_resource::Epoch;
use starcoin_vm_types::time::TimeService;
use std::collections::HashMap;

use crate::TransactionInfoWithProof;
pub use starcoin_types::block::ExecutedBlock;
use starcoin_vm_types::access_path::AccessPath;
use starcoin_vm_types::contract_event::ContractEvent;

pub struct VerifiedBlock(pub Block);
pub type MintedUncleNumber = u64;

pub trait ChainReader {
    fn info(&self) -> ChainInfo;
    fn status(&self) -> ChainStatus;
    /// Get latest block with block_info
    fn head_block(&self) -> ExecutedBlock;
    fn current_header(&self) -> BlockHeader;
    fn get_header(&self, hash: HashValue) -> Result<Option<BlockHeader>>;
    fn get_header_by_number(&self, number: BlockNumber) -> Result<Option<BlockHeader>>;
    fn get_block_by_number(&self, number: BlockNumber) -> Result<Option<Block>>;
    /// Get latest `count` blocks before `number`. if `number` is absent, use head block number.
    /// the block of `number` is inclusive.
    fn get_blocks_by_number(&self, number: Option<BlockNumber>, count: u64) -> Result<Vec<Block>>;
    fn get_block(&self, hash: HashValue) -> Result<Option<Block>>;
    /// Get block hash by block number, if not exist, return None
    fn get_hash_by_number(&self, number: BlockNumber) -> Result<Option<HashValue>>;
    fn get_transaction(&self, hash: HashValue) -> Result<Option<Transaction>>;
    /// Get transaction info by transaction's hash
    fn get_transaction_info(&self, txn_hash: HashValue) -> Result<Option<RichTransactionInfo>>;

    /// get transaction info by global index in chain.
    fn get_transaction_info_by_global_index(
        &self,
        transaction_global_index: u64,
    ) -> Result<Option<RichTransactionInfo>>;

    fn chain_state_reader(&self) -> &dyn ChainStateReader;
    fn get_block_info(&self, block_id: Option<HashValue>) -> Result<Option<BlockInfo>>;
    fn get_total_difficulty(&self) -> Result<U256>;
    fn exist_block(&self, block_id: HashValue) -> Result<bool>;
    fn epoch(&self) -> &Epoch;
    /// Get block id vec by BlockNumber, `start_number`'s block id is include.
    fn get_block_ids(
        &self,
        start_number: BlockNumber,
        reverse: bool,
        max_size: u64,
    ) -> Result<Vec<HashValue>>;
    fn get_block_info_by_number(&self, number: BlockNumber) -> Result<Option<BlockInfo>>;

    fn time_service(&self) -> &dyn TimeService;
    fn fork(&self, block_id: HashValue) -> Result<Self>
    where
        Self: Sized;
    fn epoch_uncles(&self) -> &HashMap<HashValue, MintedUncleNumber>;
    /// Find two chain's ancestor
    fn find_ancestor(&self, another: &dyn ChainReader) -> Result<Option<BlockIdAndNumber>>;
    /// Verify block header and body, base current chain, but do not verify it execute state.
    fn verify(&self, block: Block) -> Result<VerifiedBlock>;
    /// Execute block and verify it execute state, and save result base current chain, but do not change current chain.
    fn execute(&self, block: VerifiedBlock) -> Result<ExecutedBlock>;
    /// Get chain transaction infos
    fn get_transaction_infos(
        &self,
        start_index: u64,
        reverse: bool,
        max_size: u64,
    ) -> Result<Vec<RichTransactionInfo>>;

    fn get_events(&self, txn_info_id: HashValue) -> Result<Option<Vec<ContractEvent>>>;

    /// Get transaction info proof by `transaction_global_index`
    /// `block_id`: use which block header's `txn_accumulator_root` for get proof
    fn get_transaction_proof(
        &self,
        block_id: HashValue,
        transaction_global_index: u64,
        event_index: Option<u64>,
        access_path: Option<AccessPath>,
    ) -> Result<Option<TransactionInfoWithProof>>;
}

pub trait ChainWriter {
    fn can_connect(&self, executed_block: &ExecutedBlock) -> bool;
    /// Connect a executed block to current chain.
    fn connect(&mut self, executed_block: ExecutedBlock) -> Result<ExecutedBlock>;

    /// Verify, Execute and Connect block to current chain.
    fn apply(&mut self, block: Block) -> Result<ExecutedBlock>;

    fn chain_state(&mut self) -> &dyn ChainState;
}

/// `Chain` is a trait that defines a single Chain.
pub trait Chain: ChainReader + ChainWriter {}
