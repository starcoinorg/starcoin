// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::module::map_err;
use futures::future::{FutureExt, TryFutureExt};
use starcoin_config::NodeConfig;
use starcoin_crypto::HashValue;
use starcoin_rpc_api::chain::ChainApi;
use starcoin_rpc_api::types::pubsub::EventFilter;
use starcoin_rpc_api::types::{
    BlockHeaderView, BlockSummaryView, BlockView, ChainId, ChainInfoView, EpochUncleSummaryView,
    TransactionEventView, TransactionInfoView, TransactionView,
};
use starcoin_rpc_api::FutureResult;
use starcoin_traits::ChainAsyncService;
use starcoin_types::block::{BlockInfo, BlockNumber};
use starcoin_types::filter::Filter;
use starcoin_types::startup_info::ChainInfo;
use starcoin_vm_types::on_chain_resource::{EpochInfo, GlobalTimeOnChain};
use std::convert::TryInto;
use std::sync::Arc;

pub struct ChainRpcImpl<S>
where
    S: ChainAsyncService + 'static,
{
    config: Arc<NodeConfig>,
    genesis_hash: HashValue,
    service: S,
}

impl<S> ChainRpcImpl<S>
where
    S: ChainAsyncService,
{
    pub fn new(config: Arc<NodeConfig>, genesis_hash: HashValue, service: S) -> Self {
        Self {
            config,
            genesis_hash,
            service,
        }
    }
}

impl<S> ChainApi for ChainRpcImpl<S>
where
    S: ChainAsyncService,
{
    fn id(&self) -> jsonrpc_core::Result<ChainId> {
        Ok(self.config.net().id().into())
    }

    fn info(&self) -> FutureResult<ChainInfoView> {
        let service = self.service.clone();
        let chain_id = self.config.net().chain_id();
        let genesis_hash = self.genesis_hash;
        let fut = async move {
            let chain_status = service.main_status().await?;
            //TODO get chain info from chain service.
            Ok(ChainInfo::new(chain_id, genesis_hash, chain_status).into())
        };
        Box::pin(fut.boxed().map_err(map_err))
    }

    fn get_block_by_hash(&self, hash: HashValue) -> FutureResult<Option<BlockView>> {
        let service = self.service.clone();

        let fut = async move {
            let result = service.get_block_by_hash(hash).await?;
            Ok(result.map(|b| b.try_into()).transpose()?)
        }
        .map_err(map_err);

        Box::pin(fut.boxed())
    }

    fn get_block_by_number(&self, number: u64) -> FutureResult<Option<BlockView>> {
        let service = self.service.clone();

        let fut = async move {
            let result = service.main_block_by_number(number).await?;
            Ok(result.map(|b| b.try_into()).transpose()?)
        }
        .map_err(map_err);

        Box::pin(fut.boxed())
    }

    fn get_block_info_by_number(&self, number: u64) -> FutureResult<Option<BlockInfo>> {
        let service = self.service.clone();

        let fut = async move {
            let result = service.get_block_info_by_number(number).await?;
            Ok(result)
        }
        .map_err(map_err);

        Box::pin(fut.boxed())
    }

    fn get_blocks_by_number(
        &self,
        number: Option<BlockNumber>,
        count: u64,
    ) -> FutureResult<Vec<BlockView>> {
        let service = self.service.clone();
        let fut = async move {
            let block = service.main_blocks_by_number(number, count).await?;

            Ok(block
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?)
        }
        .map_err(map_err);

        Box::pin(fut.boxed())
    }

    fn get_transaction(
        &self,
        transaction_hash: HashValue,
    ) -> FutureResult<Option<TransactionView>> {
        let service = self.service.clone();
        let fut = async move {
            let transaction = service.get_transaction(transaction_hash).await?;
            match transaction {
                None => Ok(None),
                Some(t) => {
                    let block = service
                        .get_transaction_block(transaction_hash)
                        .await?
                        .ok_or_else(|| {
                            anyhow::anyhow!(
                                "cannot find block which includes the txn {}",
                                transaction_hash
                            )
                        })?;
                    TransactionView::new(t, &block).map(Some)
                }
            }
        }
        .map_err(map_err);

        Box::pin(fut.boxed())
    }

    fn get_transaction_info(
        &self,
        transaction_hash: HashValue,
    ) -> FutureResult<Option<TransactionInfoView>> {
        let service = self.service.clone();
        let fut = async move {
            let txn_info = {
                let info = service.get_transaction_info(transaction_hash).await?;
                if info.is_none() {
                    return Ok(None);
                }
                info.unwrap()
            };

            let block = service
                .get_transaction_block(transaction_hash)
                .await?
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "cannot locate the block which include txn {}",
                        transaction_hash
                    )
                })?;

            TransactionInfoView::new(txn_info, &block).map(Some)
        }
        .map_err(map_err);

        Box::pin(fut.boxed())
    }

    fn get_block_txn_infos(&self, block_hash: HashValue) -> FutureResult<Vec<TransactionInfoView>> {
        let service = self.service.clone();
        let fut = async move {
            let txn_infos = service.get_block_txn_infos(block_hash).await?;
            let block = service.get_block_by_hash(block_hash).await?;
            match block {
                None => Ok(vec![]),
                Some(block) => txn_infos
                    .into_iter()
                    .map(|info| TransactionInfoView::new(info, &block))
                    .collect::<Result<Vec<_>, _>>(),
            }
        }
        .map_err(map_err);

        Box::pin(fut.boxed())
    }

    fn get_txn_info_by_block_and_index(
        &self,
        block_hash: HashValue,
        idx: u64,
    ) -> FutureResult<Option<TransactionInfoView>> {
        let service = self.service.clone();
        let fut = async move {
            let block = service.get_block_by_hash(block_hash).await?;
            match block {
                None => Ok(None),
                Some(block) => {
                    let txn_info = service
                        .get_txn_info_by_block_and_index(block_hash, idx)
                        .await?;
                    txn_info
                        .map(|info| TransactionInfoView::new(info, &block))
                        .transpose()
                }
            }
        }
        .map_err(map_err);

        Box::pin(fut.boxed())
    }
    fn get_events_by_txn_hash(
        &self,
        txn_hash: HashValue,
    ) -> FutureResult<Vec<TransactionEventView>> {
        let service = self.service.clone();
        let fut = async move {
            let events = service.get_events_by_txn_hash(txn_hash).await?;
            Ok(events.into_iter().map(Into::into).collect())
        }
        .map_err(map_err);

        Box::pin(fut.boxed())
    }

    fn get_events(&self, mut filter: EventFilter) -> FutureResult<Vec<TransactionEventView>> {
        let service = self.service.clone();
        let config = self.config.clone();
        let fut = async move {
            if filter.to_block.is_none() {
                // if user hasn't specify the `to_block`, we use latest block as the to_block.
                let header_block_number = service.main_head_header().await?.number();
                filter.to_block = Some(header_block_number);
            }

            let filter: Filter = filter.try_into()?;

            let max_block_range = config.rpc.block_query_max_range();
            // if the from~to range is bigger than what we configured, return invalid param error.
            if filter
                .to_block
                .checked_sub(filter.from_block)
                .filter(|r| *r > max_block_range)
                .is_some()
            {
                return Err(jsonrpc_core::Error::invalid_params(format!(
                    "from_block is too far, max block range is {} ",
                    max_block_range
                ))
                .into());
            }

            service.main_events(filter).await
        }
        .map_ok(|d| d.into_iter().map(|e| e.into()).collect())
        .map_err(map_err);

        Box::pin(fut.boxed())
    }

    fn current_epoch(&self) -> FutureResult<EpochInfo> {
        let service = self.service.clone();
        let fut = async move { service.epoch_info().await };

        Box::pin(fut.boxed().map_err(map_err))
    }

    fn get_epoch_info_by_number(&self, number: BlockNumber) -> FutureResult<EpochInfo> {
        let service = self.service.clone();
        let fut = async move { service.get_epoch_info_by_number(number).await };

        Box::pin(fut.boxed().map_err(map_err))
    }

    fn get_global_time_by_number(&self, number: BlockNumber) -> FutureResult<GlobalTimeOnChain> {
        let service = self.service.clone();
        let fut = async move { service.get_global_time_by_number(number).await };

        Box::pin(fut.boxed().map_err(map_err))
    }

    fn get_epoch_uncles_by_number(
        &self,
        number: BlockNumber,
    ) -> FutureResult<Vec<BlockSummaryView>> {
        let service = self.service.clone();
        let fut = async move {
            let blocks = service.get_epoch_uncles_by_number(Some(number)).await?;
            Ok(blocks.into_iter().map(Into::into).collect())
        }
        .map_err(map_err);

        Box::pin(fut.boxed())
    }

    fn get_headers(&self, block_hashes: Vec<HashValue>) -> FutureResult<Vec<BlockHeaderView>> {
        let service = self.service.clone();
        let fut = async move {
            let headers = service.get_headers(block_hashes).await?;
            Ok(headers.into_iter().map(Into::into).collect())
        }
        .map_err(map_err);

        Box::pin(fut.boxed())
    }

    fn epoch_uncle_summary_by_number(
        &self,
        number: BlockNumber,
    ) -> FutureResult<EpochUncleSummaryView> {
        let service = self.service.clone();
        let fut = async move {
            let summary = service.epoch_uncle_summary_by_number(Some(number)).await?;
            Ok(summary.into())
        }
        .map_err(map_err);

        Box::pin(fut.boxed())
    }
}
