// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{ChainAsyncService, ConnectBlockResult};
use anyhow::{Error, Result};
use crypto::HashValue;
use types::{
    account_address::AccountAddress,
    block::{Block, BlockHeader, BlockInfo, BlockNumber, BlockState, BlockTemplate},
    contract_event::ContractEvent,
    startup_info::{ChainInfo, StartupInfo},
    transaction::{SignedUserTransaction, Transaction, TransactionInfo},
};
use starcoin_vm_types::on_chain_config::EpochInfo;

//TODO implement Mock service
#[derive(Clone, Default)]
pub struct MockChainService;

#[async_trait::async_trait]
impl ChainAsyncService for MockChainService {
    async fn try_connect(self, _block: Block) -> Result<ConnectBlockResult> {
        unimplemented!()
    }

    async fn try_connect_without_execute(&mut self, _block: Block) -> Result<ConnectBlockResult> {
        unimplemented!()
    }

    async fn get_header_by_hash(self, _hash: &HashValue) -> Result<Option<BlockHeader>> {
        unimplemented!()
    }

    async fn get_block_by_hash(self, _hash: HashValue) -> Result<Block> {
        unimplemented!()
    }

    async fn get_block_state_by_hash(self, _hash: &HashValue) -> Result<Option<BlockState>> {
        unimplemented!()
    }

    async fn get_block_info_by_hash(self, _hash: &HashValue) -> Result<Option<BlockInfo>> {
        unimplemented!()
    }

    async fn get_transaction(self, _txn_id: HashValue) -> Result<Transaction> {
        unimplemented!()
    }

    async fn get_transaction_info(self, _txn_id: HashValue) -> Result<Option<TransactionInfo>> {
        unimplemented!()
    }

    async fn get_block_txn_infos(self, _block_id: HashValue) -> Result<Vec<TransactionInfo>> {
        unimplemented!()
    }

    async fn get_txn_info_by_block_and_index(
        self,
        _block_id: HashValue,
        _idx: u64,
    ) -> Result<Option<TransactionInfo>, Error> {
        unimplemented!()
    }

    async fn get_events_by_txn_info_id(
        self,
        _txn_info_id: HashValue,
    ) -> Result<Option<Vec<ContractEvent>>, Error> {
        unimplemented!()
    }
    async fn master_head_header(self) -> Result<Option<BlockHeader>> {
        unimplemented!()
    }

    async fn master_head_block(self) -> Result<Option<Block>> {
        unimplemented!()
    }

    async fn master_block_by_number(self, _number: u64) -> Result<Block> {
        unimplemented!()
    }

    async fn master_blocks_by_number(
        self,
        _number: Option<BlockNumber>,
        _count: u64,
    ) -> Result<Vec<Block>> {
        unimplemented!()
    }

    async fn master_block_header_by_number(self, _number: BlockNumber) -> Result<BlockHeader> {
        unimplemented!()
    }

    async fn master_startup_info(self) -> Result<StartupInfo> {
        unimplemented!()
    }

    async fn master_head(self) -> Result<ChainInfo> {
        unimplemented!()
    }

    async fn epoch_info(self) -> Result<EpochInfo>{
        unimplemented!()
    }

    async fn create_block_template(
        self,
        _author: AccountAddress,
        _auth_key_prefix: Option<Vec<u8>>,
        _parent_hash: Option<HashValue>,
        _txs: Vec<SignedUserTransaction>,
    ) -> Result<BlockTemplate> {
        unimplemented!()
    }
}
