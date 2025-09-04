// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use crate::{TransactionInfoWithProof, TransactionInfoWithProof2};
use anyhow::Result;
use starcoin_crypto::HashValue;
use starcoin_state_api::ChainStateReader;
use starcoin_statedb::ChainStateDB;
use starcoin_time_service::TimeService;
use starcoin_types::block::BlockIdAndNumber;
pub use starcoin_types::block::ExecutedBlock;
use starcoin_types::startup_info::{ChainInfo, ChainStatus};
use starcoin_types::transaction::{StcRichTransactionInfo, StcTransaction};
use starcoin_types::{
    block::{Block, BlockHeader, BlockInfo, BlockNumber},
    U256,
};
use starcoin_vm2_state_api::ChainStateReader as ChainStateReader2;
use starcoin_vm2_statedb::ChainStateDB as ChainStateDB2;
use starcoin_vm2_vm_types::access_path::AccessPath as AccessPath2;
use starcoin_vm2_vm_types::on_chain_resource::Epoch;
use starcoin_vm_types::access_path::AccessPath;
use starcoin_vm_types::contract_event::ContractEvent;
use std::collections::HashMap;

use starcoin_dag::types::ghostdata::GhostdagData;
use starcoin_vm_types::genesis_config::ConsensusStrategy;

pub struct VerifiedBlock {
    pub block: Block,
    pub ghostdata: GhostdagData,
}
pub type MintedUncleNumber = u64;

pub trait ChainReader {
    fn info(&self) -> ChainInfo;
    fn status(&self) -> ChainStatus;
    /// Get latest block with block_info
    fn head_block(&self) -> ExecutedBlock;
    fn current_header(&self) -> BlockHeader;
    /// Get header by hash - WARNING: This filters out uncle/fork blocks in DAG mode!
    /// Only returns headers that exist on the main chain.
    /// Use get_header_by_hash() if you need to access all blocks including uncles.
    fn get_header(&self, hash: HashValue) -> Result<Option<BlockHeader>>;
    fn get_header_by_number(&self, number: BlockNumber) -> Result<Option<BlockHeader>>;
    fn get_block_by_number(&self, number: BlockNumber) -> Result<Option<Block>>;
    /// if  `reverse` is true , get latest `count` blocks before `number`. if `number` is absent, use head block number.
    /// if  `reverse` is false , get `count` blocks after `number` . if `number` is absent, use head block number.
    /// the block of `number` is inclusive.
    fn get_blocks_by_number(
        &self,
        number: Option<BlockNumber>,
        reverse: bool,
        count: u64,
    ) -> Result<Vec<Block>>;
    fn get_block(&self, hash: HashValue) -> Result<Option<Block>>;
    /// Get block hash by block number, if not exist, return None
    fn get_hash_by_number(&self, number: BlockNumber) -> Result<Option<HashValue>>;
    fn get_transaction(&self, hash: HashValue) -> Result<Option<StcTransaction>>;
    /// Get transaction info by transaction's hash
    fn get_transaction_info(&self, txn_hash: HashValue) -> Result<Option<StcRichTransactionInfo>>;

    /// get transaction info by global index in chain.
    fn get_transaction_info_by_global_index(
        &self,
        transaction_global_index: u64,
    ) -> Result<Option<StcRichTransactionInfo>>;

    fn chain_state_reader(&self) -> &dyn ChainStateReader;
    fn chain_state_reader2(&self) -> &dyn ChainStateReader2;
    fn get_block_info(&self, block_id: Option<HashValue>) -> Result<Option<BlockInfo>>;
    fn get_total_difficulty(&self) -> Result<U256>;
    fn exist_block(&self, block_id: HashValue) -> Result<bool>;
    fn epoch(&self) -> &Epoch;
    fn consensus_strategy(&self) -> ConsensusStrategy;
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
    fn execute(&mut self, block: VerifiedBlock) -> Result<ExecutedBlock>;
    fn execute_without_save(&self, verified_block: VerifiedBlock) -> Result<ExecutedBlock>;
    /// Get chain transaction infos
    fn get_transaction_infos(
        &self,
        start_index: u64,
        reverse: bool,
        max_size: u64,
    ) -> Result<Vec<StcRichTransactionInfo>>;

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
    /// Get transaction info proof by `transaction_global_index` using VM2 types.
    ///
    /// # Parameters
    /// - `block_id`: The ID of the block whose `txn_accumulator_root` is used to generate the proof.
    /// - `transaction_global_index`: The global index of the transaction for which the proof is requested.
    /// - `event_index`: (Optional) The index of the event within the transaction, if applicable.
    /// - `access_path`: (Optional) The access path for the resource or data being queried, using VM2's `AccessPath2` type.
    ///
    /// # Returns
    /// - `Result<Option<TransactionInfoWithProof2>>`:
    ///   - `Ok(Some(TransactionInfoWithProof2))`: The proof for the specified transaction and optional event or access path.
    ///   - `Ok(None)`: If no proof is available for the given parameters.
    ///   - `Err`: If an error occurs while generating the proof.
    fn get_transaction_proof2(
        &self,
        block_id: HashValue,
        transaction_global_index: u64,
        event_index: Option<u64>,
        access_path: Option<AccessPath2>,
    ) -> Result<Option<TransactionInfoWithProof2>>;

    // DAG methods
    fn current_tips_hash(&self, pruning_point: HashValue) -> Result<Vec<HashValue>>;
    fn has_dag_block(&self, header_id: HashValue) -> Result<bool>;
    fn calc_ghostdata_and_check_bounded_merge_depth(
        &self,
        header: &BlockHeader,
    ) -> Result<starcoin_dag::types::ghostdata::GhostdagData>;
    fn is_dag_ancestor_of(&self, ancestor: HashValue, descendant: HashValue) -> Result<bool>;
    fn get_pruning_height(&self) -> BlockNumber;
    fn get_pruning_config(&self) -> (u64, u64);
    fn get_genesis_hash(&self) -> HashValue;
    fn dag(&self) -> starcoin_dag::blockdag::BlockDAG;
    fn get_header_by_hash(&self, block_id: HashValue) -> Result<Option<BlockHeader>>;
    fn validate_pruning_point(
        &self,
        ghostdata: &starcoin_dag::types::ghostdata::GhostdagData,
        pruning_point: HashValue,
    ) -> Result<()>;
    fn check_parents_ready(&self, block_header: &BlockHeader) -> bool;
}

pub trait ChainWriter {
    fn can_connect(&self, executed_block: &ExecutedBlock) -> bool;
    /// Connect a executed block to current chain.
    fn connect(&mut self, executed_block: ExecutedBlock) -> Result<ExecutedBlock>;

    /// Verify, Execute and Connect block to current chain.
    fn apply(&mut self, block: Block) -> Result<ExecutedBlock>;

    fn chain_state(&mut self) -> &ChainStateDB;

    fn chain_state2(&mut self) -> &ChainStateDB2;

    /// Apply block for sync without full verification
    fn apply_for_sync(&mut self, block: Block) -> Result<ExecutedBlock>;
}

/// `Chain` is a trait that defines a single Chain.
pub trait Chain: ChainReader + ChainWriter {}
