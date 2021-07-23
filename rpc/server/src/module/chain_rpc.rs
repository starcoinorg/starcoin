// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::module::map_err;
use futures::future::{FutureExt, TryFutureExt};
use starcoin_abi_decoder::decode_txn_payload;
use starcoin_chain_service::ChainAsyncService;
use starcoin_config::NodeConfig;
use starcoin_crypto::HashValue;
use starcoin_resource_viewer::MoveValueAnnotator;
use starcoin_rpc_api::chain::{ChainApi, GetBlockOption, GetEventOption, GetTransactionOption};
use starcoin_rpc_api::types::pubsub::EventFilter;
use starcoin_rpc_api::types::{
    BlockHeaderView, BlockSummaryView, BlockTransactionsView, BlockView, ChainId, ChainInfoView,
    EpochUncleSummaryView, TransactionEventResponse, TransactionInfoView, TransactionView,
};
use starcoin_rpc_api::FutureResult;
use starcoin_state_api::StateView;
use starcoin_statedb::ChainStateDB;
use starcoin_storage::Storage;
use starcoin_types::block::{BlockInfo, BlockNumber};
use starcoin_types::filter::Filter;
use starcoin_types::startup_info::ChainInfo;
use starcoin_types::transaction::TransactionInfo;
use starcoin_vm_types::on_chain_resource::{EpochInfo, GlobalTimeOnChain};
use std::convert::TryInto;
use std::sync::Arc;

pub struct ChainRpcImpl<S>
where
    S: ChainAsyncService + 'static,
{
    config: Arc<NodeConfig>,
    genesis_hash: HashValue,
    storage: Arc<Storage>,
    service: S,
}

impl<S> ChainRpcImpl<S>
where
    S: ChainAsyncService,
{
    pub fn new(
        config: Arc<NodeConfig>,
        genesis_hash: HashValue,
        storage: Arc<Storage>,
        service: S,
    ) -> Self {
        Self {
            config,
            genesis_hash,
            storage,
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

    fn get_block_by_hash(
        &self,
        hash: HashValue,
        option: Option<GetBlockOption>,
    ) -> FutureResult<Option<BlockView>> {
        let service = self.service.clone();
        let decode = option.unwrap_or_default().decode;
        let storage = self.storage.clone();
        let fut = async move {
            let result = service.get_block_by_hash(hash).await?;
            let mut block: Option<BlockView> = result.map(|b| b.try_into()).transpose()?;
            if decode {
                let state = ChainStateDB::new(
                    storage,
                    Some(service.main_head_header().await?.state_root()),
                );
                if let Some(block) = block.as_mut() {
                    try_decode_block_txns(&state, block)?;
                }
            }
            Ok(block)
        }
        .map_err(map_err);

        Box::pin(fut.boxed())
    }

    fn get_block_by_number(
        &self,
        number: u64,
        option: Option<GetBlockOption>,
    ) -> FutureResult<Option<BlockView>> {
        let service = self.service.clone();
        let decode = option.unwrap_or_default().decode;
        let storage = self.storage.clone();

        let fut = async move {
            let result = service.main_block_by_number(number).await?;
            let mut block: Option<BlockView> = result.map(|b| b.try_into()).transpose()?;
            if decode {
                let state = ChainStateDB::new(
                    storage,
                    Some(service.main_head_header().await?.state_root()),
                );
                if let Some(block) = block.as_mut() {
                    try_decode_block_txns(&state, block)?;
                }
            }
            Ok(block)
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
        let config = self.config.clone();
        let fut = async move {
            let end_block_number = match number {
                Some(num) => num,
                None => service.clone().main_head_header().await?.number(),
            };

            let max_return_num = count
                .min(end_block_number + 1)
                .min(config.rpc.block_query_max_range());
            let block = service
                .main_blocks_by_number(number, max_return_num)
                .await?;

            block
                .into_iter()
                .map(|blk| BlockView::try_from_block(blk, true))
                .collect::<Result<Vec<_>, _>>()
        }
        .map_err(map_err);

        Box::pin(fut.boxed())
    }

    fn get_transaction(
        &self,
        transaction_hash: HashValue,
        option: Option<GetTransactionOption>,
    ) -> FutureResult<Option<TransactionView>> {
        let service = self.service.clone();
        let decode_payload = option.unwrap_or_default().decode;
        let storage = self.storage.clone();
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

                    let mut txn = TransactionView::new(t, &block)?;
                    if decode_payload {
                        let state = ChainStateDB::new(
                            storage,
                            Some(service.main_head_header().await?.state_root()),
                        );
                        if let Some(txn) = txn.user_transaction.as_mut() {
                            let txn_payload =
                                bcs_ext::from_bytes(txn.raw_txn.payload.0.as_slice())?;
                            txn.raw_txn.decoded_payload =
                                Some(decode_txn_payload(&state, &txn_payload)?.into());
                        }
                    }
                    Ok(Some(txn))
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
                .get_block_by_hash(txn_info.block_id())
                .await?
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "cannot find the block {}  which include txn {}",
                        txn_info.block_id(),
                        transaction_hash
                    )
                })?;

            TransactionInfoView::new(Into::<(_, TransactionInfo)>::into(txn_info).1, &block)
                .map(Some)
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
                    .map(|info| {
                        TransactionInfoView::new(Into::<(_, TransactionInfo)>::into(info).1, &block)
                    })
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
                        .map(|info| {
                            TransactionInfoView::new(
                                Into::<(_, TransactionInfo)>::into(info).1,
                                &block,
                            )
                        })
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
        option: Option<GetEventOption>,
    ) -> FutureResult<Vec<TransactionEventResponse>> {
        let event_option = option.unwrap_or_default();
        let service = self.service.clone();
        let storage = self.storage.clone();
        let fut = async move {
            let events = service.get_events_by_txn_hash(txn_hash).await?;
            let state_root = if event_option.decode {
                Some(service.main_head_header().await?.state_root())
            } else {
                None
            };

            let mut resp_data: Vec<_> = events
                .into_iter()
                .map(|e| TransactionEventResponse {
                    event: e.into(),
                    decode_event_data: None,
                })
                .collect();

            if let Some(state_root) = state_root {
                let state = ChainStateDB::new(storage, Some(state_root));
                let annotator = MoveValueAnnotator::new(&state);
                for elem in resp_data.iter_mut() {
                    elem.decode_event_data = Some(
                        annotator
                            .view_value(&elem.event.type_tag.0, elem.event.data.0.as_slice())?
                            .into(),
                    );
                }
            }
            Ok(resp_data)
        }
        .map_err(map_err);

        Box::pin(fut.boxed())
    }

    fn get_events(
        &self,
        mut filter: EventFilter,
        option: Option<GetEventOption>,
    ) -> FutureResult<Vec<TransactionEventResponse>> {
        let event_option = option.unwrap_or_default();
        let service = self.service.clone();
        let config = self.config.clone();
        let storage = self.storage.clone();
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

            let state_root = if event_option.decode {
                Some(service.main_head_header().await?.state_root())
            } else {
                None
            };
            let mut data: Vec<_> = service
                .main_events(filter)
                .await?
                .into_iter()
                .map(|e| TransactionEventResponse {
                    event: e.into(),
                    decode_event_data: None,
                })
                .collect();
            if let Some(state_root) = state_root {
                let state = ChainStateDB::new(storage, Some(state_root));
                let annotator = MoveValueAnnotator::new(&state);
                for elem in data.iter_mut() {
                    elem.decode_event_data = Some(
                        annotator
                            .view_value(&elem.event.type_tag.0, elem.event.data.0.as_slice())?
                            .into(),
                    );
                }
            }
            Ok(data)
        }
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

fn try_decode_block_txns(state: &dyn StateView, block: &mut BlockView) -> anyhow::Result<()> {
    if let BlockTransactionsView::Full(txns) = &mut block.body {
        for txn in txns.iter_mut() {
            let txn_payload = bcs_ext::from_bytes(txn.raw_txn.payload.0.as_slice())?;
            txn.raw_txn.decoded_payload = Some(decode_txn_payload(state, &txn_payload)?.into());
        }
    }
    Ok(())
}
