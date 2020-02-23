// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::chain_state::{ChainState, ChainStateReader};
use anyhow::Result;
use crypto::HashValue;
use types::{
    block::{Block, BlockHeader, BlockNumber, BlockTemplate},
    transaction::{SignedUserTransaction, Transaction, TransactionInfo},
};

#[async_trait::async_trait]
pub trait AsyncChain: Clone + std::marker::Unpin {
    async fn current_header(self) -> Option<BlockHeader>;
    async fn get_header_by_hash(self, hash: &HashValue) -> Option<BlockHeader>;
    async fn head_block(self) -> Option<Block>;
    async fn get_header_by_number(self, number: BlockNumber) -> Option<BlockHeader>;
    async fn get_block_by_number(self, number: BlockNumber) -> Option<Block>;
    async fn create_block_template(self) -> Option<BlockTemplate>;
    async fn get_block_by_hash(self, hash: &HashValue) -> Option<Block>;
}

pub trait ChainReader {
    fn head_block(&self) -> Block;
    fn current_header(&self) -> BlockHeader;
    fn get_header(&self, hash: HashValue) -> Result<Option<BlockHeader>>;
    fn get_header_by_number(&self, number: BlockNumber) -> Result<Option<BlockHeader>>;
    fn get_block_by_number(&self, number: BlockNumber) -> Result<Option<Block>>;
    fn get_block(&self, hash: HashValue) -> Result<Option<Block>>;
    fn get_transaction(&self, hash: HashValue) -> Result<Option<Transaction>>;
    /// get transaction info by transaction info hash.
    fn get_transaction_info(&self, hash: HashValue) -> Result<Option<TransactionInfo>>;
    fn create_block_template(&self, txns: Vec<SignedUserTransaction>) -> Result<BlockTemplate>;
    fn chain_state_reader(&self) -> &dyn ChainStateReader;
}

pub trait ChainWriter {
    /// execute and insert block to current chain.
    fn apply(&mut self, block: Block) -> Result<HashValue>;
    fn chain_state(&mut self) -> &dyn ChainState;
}

/// `Chain` is a trait that defines a single Chain.
pub trait Chain: ChainReader + ChainWriter {}
