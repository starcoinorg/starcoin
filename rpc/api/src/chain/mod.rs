// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

pub use self::gen_client::Client as ChainClient;
use crate::FutureResult;
use jsonrpc_derive::rpc;
use starcoin_crypto::HashValue;
use starcoin_types::block::{Block, BlockNumber};
use starcoin_types::startup_info::ChainInfo;
use starcoin_types::transaction::{Transaction, TransactionInfo};

#[rpc]
pub trait ChainApi {
    /// Get chain head info
    #[rpc(name = "chain.head")]
    fn head(&self) -> FutureResult<ChainInfo>;
    /// Get chain block info
    #[rpc(name = "chain.get_block_by_hash")]
    fn get_block_by_hash(&self, hash: HashValue) -> FutureResult<Block>;
    /// Get chain blocks by number
    #[rpc(name = "chain.get_block_by_number")]
    fn get_block_by_number(&self, number: BlockNumber) -> FutureResult<Block>;
    /// Get latest `count` blocks before `number`. if `number` is absent, use head block number.
    #[rpc(name = "chain.get_blocks_by_number")]
    fn get_blocks_by_number(
        &self,
        number: Option<BlockNumber>,
        count: u64,
    ) -> FutureResult<Vec<Block>>;
    /// Get chain transactions
    #[rpc(name = "chain.get_transaction")]
    fn get_transaction(&self, transaction_id: HashValue) -> FutureResult<Transaction>;
    /// Get chain transactions
    #[rpc(name = "chain.get_transaction_info")]
    fn get_transaction_info(&self, transaction_id: HashValue)
        -> FutureResult<Vec<TransactionInfo>>;

    /// Get chain transactions infos by block id
    #[rpc(name = "chain.get_block_txn_infos")]
    fn get_txn_by_block(&self, block_id: HashValue) -> FutureResult<Vec<TransactionInfo>>;

    /// Get txn info of a txn at `idx` of block `block_id`
    #[rpc(name = "chain.get_txn_info_by_block_and_index")]
    fn get_txn_info_by_block_and_index(
        &self,
        block_id: HashValue,
        idx: u64,
    ) -> FutureResult<Option<TransactionInfo>>;

    /// Get branches of current chain, first is master.
    #[rpc(name = "chain.branches")]
    fn branches(&self) -> FutureResult<Vec<ChainInfo>>;
}
