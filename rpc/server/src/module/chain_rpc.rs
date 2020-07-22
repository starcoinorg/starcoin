// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::module::map_err;
use futures::future::{FutureExt, TryFutureExt};
use starcoin_consensus::dev::DummyHeader;
use starcoin_crypto::HashValue;
use starcoin_rpc_api::chain::ChainApi;
use starcoin_rpc_api::FutureResult;
use starcoin_traits::ChainAsyncService;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::block::{Block, BlockNumber};
use starcoin_types::contract_event::ContractEvent;
use starcoin_types::startup_info::ChainInfo;
use starcoin_types::transaction::{Transaction, TransactionInfo};
use starcoin_vm_types::on_chain_config::EpochInfo;

pub struct ChainRpcImpl<S>
where
    S: ChainAsyncService + 'static,
{
    service: S,
}

impl<S> ChainRpcImpl<S>
where
    S: ChainAsyncService,
{
    pub fn new(service: S) -> Self {
        Self { service }
    }
}

impl<S> ChainApi for ChainRpcImpl<S>
where
    S: ChainAsyncService,
{
    fn head(&self) -> FutureResult<ChainInfo> {
        let fut = self.service.clone().master_head();
        Box::new(fut.map_err(map_err).compat())
    }

    fn get_block_by_hash(&self, hash: HashValue) -> FutureResult<Block> {
        let fut = self
            .service
            .clone()
            .get_block_by_hash(hash)
            .map_err(map_err);
        Box::new(fut.compat())
    }

    fn get_block_by_number(&self, number: u64) -> FutureResult<Block> {
        let fut = self
            .service
            .clone()
            .master_block_by_number(number)
            .map_err(map_err);
        Box::new(fut.compat())
    }

    fn get_block_by_uncle(&self, uncle_id: HashValue) -> FutureResult<Option<Block>> {
        let service = self.service.clone();
        let fut = async move {
            let block = service.clone().master_block_by_uncle(uncle_id).await?;
            Ok(block)
        }
        .map_err(map_err);

        Box::new(fut.boxed().compat())
    }

    fn get_blocks_by_number(
        &self,
        number: Option<BlockNumber>,
        count: u64,
    ) -> FutureResult<Vec<Block>> {
        let fut = self
            .service
            .clone()
            .master_blocks_by_number(number, count)
            .map_err(map_err);
        Box::new(fut.compat())
    }

    fn get_transaction(&self, transaction_hash: HashValue) -> FutureResult<Transaction> {
        let fut = self
            .service
            .clone()
            .get_transaction(transaction_hash)
            .map_err(map_err);
        Box::new(fut.compat())
    }

    fn get_transaction_info(
        &self,
        transaction_hash: HashValue,
    ) -> FutureResult<Option<TransactionInfo>> {
        let fut = self
            .service
            .clone()
            .get_transaction_info(transaction_hash)
            .map_err(map_err);
        Box::new(fut.compat())
    }

    fn get_txn_by_block(&self, block_id: HashValue) -> FutureResult<Vec<TransactionInfo>> {
        let fut = self
            .service
            .clone()
            .get_block_txn_infos(block_id)
            .map_err(map_err);
        Box::new(fut.compat())
    }
    fn get_txn_info_by_block_and_index(
        &self,
        block_id: HashValue,
        idx: u64,
    ) -> FutureResult<Option<TransactionInfo>> {
        let fut = self
            .service
            .clone()
            .get_txn_info_by_block_and_index(block_id, idx)
            .map_err(map_err);
        Box::new(fut.compat())
    }
    fn get_events_by_txn_info_id(
        &self,
        txn_info_id: HashValue,
    ) -> FutureResult<Vec<ContractEvent>> {
        let fut = self
            .service
            .clone()
            .get_events_by_txn_info_id(txn_info_id)
            .map_ok(|d| d.unwrap_or_default())
            .map_err(map_err);
        Box::new(fut.compat())
    }
    fn branches(&self) -> FutureResult<Vec<ChainInfo>> {
        let fut = self
            .service
            .clone()
            .master_startup_info()
            .map(|result| Ok(Into::<Vec<ChainInfo>>::into(result?)))
            .map_err(map_err);
        Box::new(fut.compat())
    }

    fn current_epoch(&self) -> FutureResult<EpochInfo> {
        let fut = self.service.clone().epoch_info().map_err(map_err);

        Box::new(fut.compat())
    }

    fn create_dev_block(
        &self,
        author: AccountAddress,
        auth_key_prefix: Vec<u8>,
        parent_id: Option<HashValue>,
    ) -> FutureResult<HashValue> {
        let service = self.service.clone();
        let fut = async move {
            let block_template = service
                .clone()
                .create_block_template(author, Some(auth_key_prefix), parent_id, Vec::new())
                .await?;

            let block = block_template.into_block(DummyHeader {}, 10000.into());
            let block_id = block.id();

            let _ = service.clone().try_connect(block).await?;

            Ok(block_id)
        }
        .map_err(map_err);

        Box::new(fut.boxed().compat())
    }
}
