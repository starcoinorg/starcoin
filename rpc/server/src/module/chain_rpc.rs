// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::module::map_err;
use futures::future::{FutureExt, TryFutureExt};
use starcoin_crypto::HashValue;
use starcoin_rpc_api::chain::ChainApi;
use starcoin_rpc_api::FutureResult;
use starcoin_traits::ChainAsyncService;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::block::{Block, BlockNumber};
use starcoin_types::contract_event::ContractEvent;
use starcoin_types::startup_info::ChainInfo;
use starcoin_types::transaction::{Transaction, TransactionInfo};
use starcoin_vm_types::on_chain_config::{EpochInfo, GlobalTimeOnChain};

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
        let fut = async move { service.master_head().await };
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
            let result = service.master_block_by_number(number).await?;
            Ok(result)
        }
        .map_err(map_err);

        Box::new(fut.boxed().compat())
    }

    fn get_block_by_uncle(&self, uncle_id: HashValue) -> FutureResult<Option<Block>> {
        let service = self.service.clone();
        let fut = async move {
            let block = service.master_block_by_uncle(uncle_id).await?;
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
        let service = self.service.clone();
        let fut = async move {
            let block = service.master_blocks_by_number(number, count).await?;
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

    fn get_transaction_hex(&self, transaction_id: String) -> FutureResult<Transaction> {
        let txn_hash = match HashValue::from_hex(&transaction_id) {
            Ok(t) => t,
            Err(e) => return Box::new(jsonrpc_core::futures::failed(map_err(e))),
        };
        let service = self.service.clone();
        let fut = async move {
            let block = service.get_transaction(txn_hash).await?;
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
    fn branches(&self) -> FutureResult<Vec<ChainInfo>> {
        let service = self.service.clone();
        let fut = async move { service.master_startup_info().await };

        Box::new(
            fut.boxed()
                .map(|result| Ok(Into::<Vec<ChainInfo>>::into(result?)))
                .map_err(map_err)
                .compat(),
        )
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

    fn create_dev_block(
        &self,
        author: AccountAddress,
        auth_key_prefix: Vec<u8>,
        parent_id: Option<HashValue>,
        head: bool,
    ) -> FutureResult<HashValue> {
        let service = self.service.clone();
        let fut = async move {
            let old_head = service
                .clone()
                .master_head_header()
                .await?
                .expect("head is none.");

            let p_id = match parent_id {
                Some(id) => id,
                None => old_head.parent_hash(),
            };

            let parent_number = service
                .clone()
                .get_block_by_hash(p_id)
                .await?
                .header()
                .number();
            let brother_txns = if old_head.number() > parent_number {
                service
                    .clone()
                    .master_block_by_number(parent_number + 1)
                    .await?
                    .body
                    .into()
            } else {
                Vec::new()
            };

            let block_template = service
                .clone()
                .create_block_template(author, Some(auth_key_prefix), Some(p_id), brother_txns)
                .await?;

            let difficulty = if head {
                let head_difficulty = service
                    .clone()
                    .get_block_info_by_hash(&old_head.id())
                    .await?
                    .expect("head block info is none.")
                    .get_total_difficulty();
                let parent_difficulty = service
                    .clone()
                    .get_block_info_by_hash(&p_id)
                    .await?
                    .expect("parent block info is none.")
                    .get_total_difficulty();

                if head_difficulty > parent_difficulty {
                    (head_difficulty - parent_difficulty) * 2
                } else {
                    1.into()
                }
            } else {
                1.into()
            };

            let block = block_template.into_block(0, difficulty);
            let block_id = block.id();

            //let _ = service.clone().try_connect(block).await?;

            Ok(block_id)
        }
        .map_err(map_err);

        Box::new(fut.boxed().compat())
    }
}
