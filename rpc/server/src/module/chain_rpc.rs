// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::module::map_err;
use futures::future::{FutureExt, TryFutureExt};
use starcoin_crypto::HashValue;
use starcoin_rpc_api::chain::ChainApi;
use starcoin_rpc_api::types::pubsub::{Event, EventFilter};
use starcoin_rpc_api::FutureResult;
use starcoin_traits::ChainAsyncService;
use starcoin_types::block::{Block, BlockHeader, BlockNumber, BlockSummary, EpochUncleSummary};
use starcoin_types::contract_event::ContractEvent;
use starcoin_types::startup_info::ChainInfo;
use starcoin_types::stress_test::TPS;
use starcoin_types::transaction::{Transaction, TransactionInfo};
use starcoin_vm_types::on_chain_resource::{EpochInfo, GlobalTimeOnChain};
use std::convert::TryInto;

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
        let service = self.service.clone();
        let fut = async move { service.main_head().await };
        Box::new(fut.boxed().map_err(map_err).compat())
    }

    fn get_block_by_hash(&self, hash: HashValue) -> FutureResult<Block> {
        let service = self.service.clone();

        let fut = async move {
            let result = service.get_block_by_hash(hash).await?;
            Ok(result)
        }
        .map_err(map_err);

        Box::new(fut.boxed().compat())
    }

    fn get_block_by_number(&self, number: u64) -> FutureResult<Block> {
        let service = self.service.clone();

        let fut = async move {
            let result = service.main_block_by_number(number).await?;
            Ok(result)
        }
        .map_err(map_err);

        Box::new(fut.boxed().compat())
    }

    fn get_blocks_by_number(
        &self,
        number: Option<BlockNumber>,
        count: u64,
    ) -> FutureResult<Vec<Block>> {
        let service = self.service.clone();
        let fut = async move {
            let block = service.main_blocks_by_number(number, count).await?;
            Ok(block)
        }
        .map_err(map_err);

        Box::new(fut.boxed().compat())
    }

    fn get_transaction(&self, transaction_hash: HashValue) -> FutureResult<Transaction> {
        let service = self.service.clone();
        let fut = async move {
            let block = service.get_transaction(transaction_hash).await?;
            Ok(block)
        }
        .map_err(map_err);

        Box::new(fut.boxed().compat())
    }

    fn get_transaction_info(
        &self,
        transaction_hash: HashValue,
    ) -> FutureResult<Option<TransactionInfo>> {
        let service = self.service.clone();
        let fut = async move {
            let block = service.get_transaction_info(transaction_hash).await?;
            Ok(block)
        }
        .map_err(map_err);

        Box::new(fut.boxed().compat())
    }

    fn get_txn_by_block(&self, block_id: HashValue) -> FutureResult<Vec<TransactionInfo>> {
        let service = self.service.clone();
        let fut = async move {
            let block = service.get_block_txn_infos(block_id).await?;
            Ok(block)
        }
        .map_err(map_err);

        Box::new(fut.boxed().compat())
    }

    fn get_txn_info_by_block_and_index(
        &self,
        block_id: HashValue,
        idx: u64,
    ) -> FutureResult<Option<TransactionInfo>> {
        let service = self.service.clone();
        let fut = async move {
            let block = service
                .get_txn_info_by_block_and_index(block_id, idx)
                .await?;
            Ok(block)
        }
        .map_err(map_err);

        Box::new(fut.boxed().compat())
    }
    fn get_events_by_txn_info_id(
        &self,
        txn_info_id: HashValue,
    ) -> FutureResult<Vec<ContractEvent>> {
        let service = self.service.clone();
        let fut = async move { service.get_events_by_txn_info_id(txn_info_id).await }
            .map_ok(|d| d.unwrap_or_default())
            .map_err(map_err);

        Box::new(fut.boxed().compat())
    }

    fn get_events(&self, filter: EventFilter) -> FutureResult<Vec<Event>> {
        let service = self.service.clone();
        let fut = async move {
            let filter = filter.try_into()?;
            service.main_events(filter).await
        }
        .map_ok(|d| d.into_iter().map(|e| e.into()).collect())
        .map_err(map_err);

        Box::new(fut.boxed().compat())
    }

    fn current_epoch(&self) -> FutureResult<EpochInfo> {
        let service = self.service.clone();
        let fut = async move { service.epoch_info().await };

        Box::new(fut.boxed().map_err(map_err).compat())
    }

    fn get_epoch_info_by_number(&self, number: BlockNumber) -> FutureResult<EpochInfo> {
        let service = self.service.clone();
        let fut = async move { service.get_epoch_info_by_number(number).await };

        Box::new(fut.boxed().map_err(map_err).compat())
    }

    fn get_global_time_by_number(&self, number: BlockNumber) -> FutureResult<GlobalTimeOnChain> {
        let service = self.service.clone();
        let fut = async move { service.get_global_time_by_number(number).await };

        Box::new(fut.boxed().map_err(map_err).compat())
    }

    fn get_block_by_uncle(&self, uncle_id: HashValue) -> FutureResult<Option<Block>> {
        let service = self.service.clone();
        let fut = async move {
            let block = service.main_block_by_uncle(uncle_id).await?;
            Ok(block)
        }
        .map_err(map_err);

        Box::new(fut.boxed().compat())
    }

    fn tps(&self, number: Option<BlockNumber>) -> FutureResult<TPS> {
        let service = self.service.clone();
        let fut = async move { service.tps(number).await };

        Box::new(fut.boxed().map_err(map_err).compat())
    }

    fn get_epoch_uncles_by_number(&self, number: BlockNumber) -> FutureResult<Vec<BlockSummary>> {
        let service = self.service.clone();
        let fut = async move {
            let blocks = service.get_epoch_uncles_by_number(Some(number)).await?;
            Ok(blocks)
        }
        .map_err(map_err);

        Box::new(fut.boxed().compat())
    }

    fn epoch_uncle_summary_by_number(
        &self,
        number: BlockNumber,
    ) -> FutureResult<EpochUncleSummary> {
        let service = self.service.clone();
        let fut = async move {
            let summary = service.epoch_uncle_summary_by_number(Some(number)).await?;
            Ok(summary)
        }
        .map_err(map_err);

        Box::new(fut.boxed().compat())
    }

    fn get_headers(&self, ids: Vec<HashValue>) -> FutureResult<Vec<BlockHeader>> {
        let service = self.service.clone();
        let fut = async move { Ok(service.get_headers(ids).await?) }.map_err(map_err);

        Box::new(fut.boxed().compat())
    }

    fn uncle_path(
        &self,
        block_id: HashValue,
        uncle_id: HashValue,
    ) -> FutureResult<Vec<BlockHeader>> {
        let service = self.service.clone();
        let fut = async move { Ok(service.uncle_path(block_id, uncle_id).await?) }.map_err(map_err);

        Box::new(fut.boxed().compat())
    }
}
