// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::ConnectBlockResult;
use anyhow::Result;
use starcoin_crypto::HashValue;
use starcoin_state_api::ChainStateReader;
use starcoin_types::block::BlockState;
use starcoin_types::contract_event::ContractEvent;
use starcoin_types::peer_info::PeerId;
use starcoin_types::startup_info::ChainInfo;
use starcoin_types::transaction::{Transaction, TransactionInfo};
use starcoin_types::{
    account_address::AccountAddress,
    block::{Block, BlockHeader, BlockInfo, BlockNumber, BlockTemplate},
    startup_info::StartupInfo,
    transaction::SignedUserTransaction,
};
use starcoin_vm_types::on_chain_config::{EpochInfo, GlobalTimeOnChain};

/// implement ChainService
pub trait ChainService {
    /// chain service
    fn try_connect(&mut self, block: Block) -> Result<ConnectBlockResult>;
    fn try_connect_without_execute(
        &mut self,
        block: Block,
        remote_chain_state: &dyn ChainStateReader,
    ) -> Result<ConnectBlockResult>;

    fn get_header_by_hash(&self, hash: HashValue) -> Result<Option<BlockHeader>>;
    fn get_block_by_hash(&self, hash: HashValue) -> Result<Option<Block>>;
    fn get_block_state_by_hash(&self, hash: HashValue) -> Result<Option<BlockState>>;
    fn get_block_info_by_hash(&self, hash: HashValue) -> Result<Option<BlockInfo>>;
    fn get_transaction(&self, hash: HashValue) -> Result<Option<Transaction>>;
    fn get_transaction_info(&self, txn_hash: HashValue) -> Result<Option<TransactionInfo>>;
    fn get_block_txn_infos(&self, block_id: HashValue) -> Result<Vec<TransactionInfo>>;
    fn get_txn_info_by_block_and_index(
        &self,
        block_id: HashValue,
        idx: u64,
    ) -> Result<Option<TransactionInfo>>;
    fn get_events_by_txn_info_id(
        &self,
        txn_info_id: HashValue,
    ) -> Result<Option<Vec<ContractEvent>>>;
    /// for master
    fn master_head_header(&self) -> BlockHeader;
    fn master_head_block(&self) -> Block;
    fn master_block_by_number(&self, number: BlockNumber) -> Result<Option<Block>>;
    fn master_block_by_uncle(&self, uncle_id: HashValue) -> Result<Option<Block>>;
    fn master_block_header_by_number(&self, number: BlockNumber) -> Result<Option<BlockHeader>>;
    fn master_startup_info(&self) -> StartupInfo;
    fn master_blocks_by_number(
        &self,
        number: Option<BlockNumber>,
        count: u64,
    ) -> Result<Vec<Block>>;
    fn epoch_info(&self) -> Result<EpochInfo>;
    fn get_epoch_info_by_number(&self, number: BlockNumber) -> Result<EpochInfo>;
    fn get_global_time_by_number(&self, number: BlockNumber) -> Result<GlobalTimeOnChain>;

    /// just for test
    fn create_block_template(
        &self,
        author: AccountAddress,
        auth_key_prefix: Option<Vec<u8>>,
        parent_hash: Option<HashValue>,
        user_txns: Vec<SignedUserTransaction>,
    ) -> Result<BlockTemplate>;
}

/// ChainActor
#[async_trait::async_trait]
pub trait ChainAsyncService:
    Clone + std::marker::Unpin + std::marker::Sync + std::marker::Send
{
    /// chain service
    async fn try_connect(&self, block: Block) -> Result<ConnectBlockResult>;
    async fn try_connect_without_execute(
        &mut self,
        block: Block,
        peer_id: PeerId,
    ) -> Result<ConnectBlockResult>;
    async fn get_header_by_hash(&self, hash: &HashValue) -> Result<Option<BlockHeader>>;
    async fn get_block_by_hash(&self, hash: HashValue) -> Result<Block>;
    async fn get_block_state_by_hash(&self, hash: &HashValue) -> Result<Option<BlockState>>;
    async fn get_block_info_by_hash(&self, hash: &HashValue) -> Result<Option<BlockInfo>>;
    async fn get_transaction(&self, txn_hash: HashValue) -> Result<Transaction>;
    async fn get_transaction_info(&self, txn_hash: HashValue) -> Result<Option<TransactionInfo>>;
    async fn get_block_txn_infos(&self, block_id: HashValue) -> Result<Vec<TransactionInfo>>;
    async fn get_txn_info_by_block_and_index(
        &self,
        block_id: HashValue,
        idx: u64,
    ) -> Result<Option<TransactionInfo>>;
    async fn get_events_by_txn_info_id(
        &self,
        txn_info_id: HashValue,
    ) -> Result<Option<Vec<ContractEvent>>>;
    /// for master
    async fn master_head_header(&self) -> Result<Option<BlockHeader>>;
    async fn master_head_block(&self) -> Result<Option<Block>>;
    async fn master_block_by_number(&self, number: BlockNumber) -> Result<Block>;
    async fn master_block_by_uncle(&self, uncle_id: HashValue) -> Result<Option<Block>>;
    async fn master_blocks_by_number(
        &self,
        number: Option<BlockNumber>,
        count: u64,
    ) -> Result<Vec<Block>>;
    async fn master_block_header_by_number(&self, number: BlockNumber) -> Result<BlockHeader>;
    async fn master_startup_info(&self) -> Result<StartupInfo>;
    async fn master_head(&self) -> Result<ChainInfo>;
    async fn epoch_info(&self) -> Result<EpochInfo>;
    async fn get_epoch_info_by_number(&self, number: BlockNumber) -> Result<EpochInfo>;
    async fn get_global_time_by_number(&self, number: BlockNumber) -> Result<GlobalTimeOnChain>;

    /// just for test
    async fn create_block_template(
        &self,
        author: AccountAddress,
        auth_key_prefix: Option<Vec<u8>>,
        parent_hash: Option<HashValue>,
        txs: Vec<SignedUserTransaction>,
    ) -> Result<BlockTemplate>;
}
