// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use starcoin_crypto::HashValue;
use starcoin_state_api::{ChainState, ChainStateReader};
use starcoin_types::{
    account_address::AccountAddress,
    block::{Block, BlockHeader, BlockInfo, BlockNumber, BlockTemplate},
    startup_info::ChainInfo,
    transaction::{SignedUserTransaction, Transaction, TransactionInfo},
    U512,
};

pub trait ChainReader {
    fn head_block(&self) -> Block;
    fn current_header(&self) -> BlockHeader;
    fn get_header(&self, hash: HashValue) -> Result<Option<BlockHeader>>;
    fn get_header_by_number(&self, number: BlockNumber) -> Result<Option<BlockHeader>>;
    fn get_block_by_number(&self, number: BlockNumber) -> Result<Option<Block>>;
    /// Get latest `count` blocks before `number`. if `number` is absent, use head block number.
    fn get_blocks_by_number(&self, number: Option<BlockNumber>, count: u64) -> Result<Vec<Block>>;
    fn get_block(&self, hash: HashValue) -> Result<Option<Block>>;
    fn get_block_transactions(&self, block_id: HashValue) -> Result<Vec<TransactionInfo>>;
    fn get_transaction(&self, hash: HashValue) -> Result<Option<Transaction>>;
    /// get transaction info by transaction info hash.
    fn get_transaction_info(&self, hash: HashValue) -> Result<Option<TransactionInfo>>;
    fn create_block_template(
        &self,
        author: AccountAddress,
        auth_key_prefix: Option<Vec<u8>>,
        parent_hash: Option<HashValue>,
        user_txns: Vec<SignedUserTransaction>,
    ) -> Result<BlockTemplate>;
    fn chain_state_reader(&self) -> &dyn ChainStateReader;
    fn gen_tx(&self) -> Result<()>;
    fn get_chain_info(&self) -> ChainInfo;
    fn get_block_info(&self, block_id: Option<HashValue>) -> Result<Option<BlockInfo>>;
    fn get_total_difficulty(&self) -> Result<U512>;
    fn exist_block(&self, block_id: HashValue) -> bool;
}

pub trait ChainWriter {
    /// execute and insert block to current chain.
    fn apply(&mut self, block: Block) -> Result<bool>;
    /// execute and insert block to current chain.
    fn commit(&mut self, block: Block, block_info: BlockInfo) -> Result<()>;
    fn save(&mut self, block_id: HashValue, transactions: Vec<Transaction>) -> Result<()>;
    fn chain_state(&mut self) -> &dyn ChainState;
}

/// `Chain` is a trait that defines a single Chain.
pub trait Chain: ChainReader + ChainWriter {}
