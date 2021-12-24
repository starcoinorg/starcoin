// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::module::map_err;
use futures::future::{FutureExt, TryFutureExt};
use starcoin_abi_decoder::decode_txn_payload;
use starcoin_chain_service::ChainAsyncService;
use starcoin_config::NodeConfig;
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::*;
use starcoin_resource_viewer::MoveValueAnnotator;
use starcoin_rpc_api::chain::{ChainApi, GetBlockOption, GetEventOption, GetTransactionOption};
use starcoin_rpc_api::types::pubsub::EventFilter;
use starcoin_rpc_api::types::{
    BlockHeaderView, BlockInfoView, BlockTransactionsView, BlockView, ChainId, ChainInfoView,
    SignedUserTransactionView, StrView, TransactionEventResponse, TransactionInfoView,
    TransactionInfoWithProofView, TransactionView,
};
use starcoin_rpc_api::FutureResult;
use starcoin_state_api::StateView;
use starcoin_statedb::ChainStateDB;
use starcoin_storage::Storage;
use starcoin_types::access_path::AccessPath;
use starcoin_types::block::BlockNumber;
use starcoin_types::filter::Filter;
use starcoin_types::startup_info::ChainInfo;
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

    fn get_block_info_by_number(&self, number: u64) -> FutureResult<Option<BlockInfoView>> {
        let service = self.service.clone();

        let fut = async move {
            let result = service
                .get_block_info_by_number(number)
                .await?
                .map(Into::into);
            Ok(result)
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
                            try_decode_txn_payload(&state, txn)?;
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
            Ok(service
                .get_transaction_info(transaction_hash)
                .await?
                .map(Into::into))
        }
        .map_err(map_err);

        Box::pin(fut.boxed())
    }

    fn get_block_txn_infos(&self, block_hash: HashValue) -> FutureResult<Vec<TransactionInfoView>> {
        let service = self.service.clone();
        let fut = async move {
            Ok(service
                .get_block_txn_infos(block_hash)
                .await?
                .into_iter()
                .map(Into::into)
                .collect::<Vec<_>>())
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
            Ok(service
                .get_txn_info_by_block_and_index(block_hash, idx)
                .await?
                .map(Into::into))
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

    fn get_headers(&self, block_hashes: Vec<HashValue>) -> FutureResult<Vec<BlockHeaderView>> {
        let service = self.service.clone();
        let fut = async move {
            let headers = service.get_headers(block_hashes).await?;
            Ok(headers.into_iter().map(Into::into).collect())
        }
        .map_err(map_err);

        Box::pin(fut.boxed())
    }

    fn get_transaction_infos(
        &self,
        start_global_index: u64,
        reverse: bool,
        max_size: u64,
    ) -> FutureResult<Vec<TransactionInfoView>> {
        let service = self.service.clone();
        let config = self.config.clone();
        let fut = async move {
            let max_return_num = max_size.min(config.rpc.txn_info_query_max_range());
            Ok(service
                .get_transaction_infos(start_global_index, reverse, max_return_num)
                .await?
                .into_iter()
                .map(Into::into)
                .collect::<Vec<_>>())
        }
        .map_err(map_err);

        Box::pin(fut.boxed())
    }

    fn get_transaction_proof(
        &self,
        block_hash: HashValue,
        transaction_global_index: u64,
        event_index: Option<u64>,
        access_path: Option<StrView<AccessPath>>,
    ) -> FutureResult<Option<TransactionInfoWithProofView>> {
        let service = self.service.clone();
        let fut = async move {
            Ok(service
                .get_transaction_proof(
                    block_hash,
                    transaction_global_index,
                    event_index,
                    access_path.map(Into::into),
                )
                .await?
                .map(Into::into))
        }
        .map_err(map_err);

        Box::pin(fut.boxed())
    }
}

fn try_decode_block_txns(state: &dyn StateView, block: &mut BlockView) -> anyhow::Result<()> {
    if let BlockTransactionsView::Full(txns) = &mut block.body {
        for txn in txns.iter_mut() {
            try_decode_txn_payload(state, txn)?;
        }
    }
    Ok(())
}

fn try_decode_txn_payload(
    state: &dyn StateView,
    txn: &mut SignedUserTransactionView,
) -> anyhow::Result<()> {
    let txn_payload = bcs_ext::from_bytes(txn.raw_txn.payload.0.as_slice())?;
    match decode_txn_payload(state, &txn_payload) {
        // ignore decode failure, as txns may has invalid payload here.
        Err(e) => {
            debug!(
                "decode payload of txn {} failure, {:?}",
                txn.transaction_hash, e
            );
        }
        Ok(d) => {
            txn.raw_txn.decoded_payload = Some(d.into());
        }
    }
    Ok(())
}
