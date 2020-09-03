// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod chain;

pub use chain::BlockChain;

pub mod chain_service;
pub mod message;

pub use chain_service::ChainServiceImpl;

use crate::message::ChainResponse;
use actix::prelude::*;
use anyhow::{bail, format_err, Error, Result};
use crypto::HashValue;
use logger::prelude::*;
use message::ChainRequest;
use starcoin_config::NodeConfig;
use starcoin_traits::{ChainAsyncService, ReadableChainService};
use starcoin_types::{
    account_address::AccountAddress,
    block::{Block, BlockHeader, BlockInfo, BlockNumber, BlockState, BlockTemplate},
    contract_event::ContractEvent,
    startup_info::{ChainInfo, StartupInfo},
    transaction::{SignedUserTransaction, Transaction, TransactionInfo},
};
use starcoin_vm_types::on_chain_config::{EpochInfo, GlobalTimeOnChain};
use std::sync::Arc;
use storage::Store;
use txpool::TxPoolService;

/// actor for block chain.
pub struct ChainActor {
    service: ChainServiceImpl<TxPoolService>,
}

impl ChainActor {
    pub fn launch(
        config: Arc<NodeConfig>,
        startup_info: StartupInfo,
        storage: Arc<dyn Store>,
        txpool: TxPoolService,
    ) -> Result<ChainActorRef> {
        let actor = ChainActor {
            service: ChainServiceImpl::new(config, startup_info, storage, txpool)?,
        }
        .start();
        Ok(actor.into())
    }
}

impl Actor for ChainActor {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        info!("ChainActor started");
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!("ChainActor stopped");
    }
}

impl Handler<ChainRequest> for ChainActor {
    type Result = Result<ChainResponse>;

    fn handle(&mut self, msg: ChainRequest, _ctx: &mut Self::Context) -> Self::Result {
        match msg {
            ChainRequest::CurrentHeader() => Ok(ChainResponse::BlockHeader(Box::new(Some(
                self.service.master_head_header(),
            )))),
            ChainRequest::GetHeaderByHash(hash) => Ok(ChainResponse::BlockHeader(Box::new(
                self.service.get_header_by_hash(hash)?,
            ))),
            ChainRequest::HeadBlock() => Ok(ChainResponse::Block(Box::new(
                self.service.master_head_block(),
            ))),
            ChainRequest::GetBlockByNumber(number) => Ok(ChainResponse::Block(Box::new(
                self.service
                    .master_block_by_number(number)?
                    .ok_or_else(|| {
                        format_err!("Can not find block from master by number {:?}", number)
                    })?,
            ))),
            ChainRequest::GetBlockHeaderByNumber(number) => {
                Ok(ChainResponse::BlockHeader(Box::new(Some(
                    self.service
                        .master_block_header_by_number(number)?
                        .ok_or_else(|| {
                            format_err!(
                                "Can not find block header from master by number {:?}",
                                number
                            )
                        })?,
                ))))
            }
            ChainRequest::CreateBlockTemplate(author, auth_key_prefix, parent_hash, txs) => Ok(
                ChainResponse::BlockTemplate(Box::new(self.service.create_block_template(
                    author,
                    auth_key_prefix,
                    parent_hash,
                    txs,
                )?)),
            ),
            ChainRequest::GetBlockByHash(hash) => Ok(ChainResponse::OptionBlock(
                if let Some(block) = self.service.get_block_by_hash(hash)? {
                    Some(Box::new(block))
                } else {
                    None
                },
            )),
            ChainRequest::GetBlockByUncle(uncle_id) => Ok(ChainResponse::OptionBlock(
                if let Some(block) = self.service.master_block_by_uncle(uncle_id)? {
                    Some(Box::new(block))
                } else {
                    None
                },
            )),
            ChainRequest::GetBlockStateByHash(hash) => Ok(ChainResponse::BlockState(
                if let Some(block_state) = self.service.get_block_state_by_hash(hash)? {
                    Some(Box::new(block_state))
                } else {
                    None
                },
            )),
            ChainRequest::GetBlockInfoByHash(hash) => Ok(ChainResponse::OptionBlockInfo(Box::new(
                self.service.get_block_info_by_hash(hash)?,
            ))),
            ChainRequest::GetStartupInfo() => Ok(ChainResponse::StartupInfo(
                self.service.master_startup_info(),
            )),
            ChainRequest::GetHeadChainInfo() => Ok(ChainResponse::ChainInfo(ChainInfo::new(
                *self.service.master_startup_info().get_master(),
            ))),
            ChainRequest::GetTransaction(hash) => Ok(ChainResponse::Transaction(Box::new(
                self.service
                    .get_transaction(hash)?
                    .ok_or_else(|| format_err!("Can not find transaction by hash {:?}", hash))?,
            ))),
            ChainRequest::GetTransactionInfo(hash) => Ok(ChainResponse::TransactionInfo(
                self.service.get_transaction_info(hash)?,
            )),
            ChainRequest::GetBlocksByNumber(number, count) => Ok(ChainResponse::VecBlock(
                self.service.master_blocks_by_number(number, count)?,
            )),
            ChainRequest::GetBlockTransactionInfos(block_id) => Ok(
                ChainResponse::TransactionInfos(self.service.get_block_txn_infos(block_id)?),
            ),
            ChainRequest::GetTransactionInfoByBlockAndIndex { block_id, txn_idx } => {
                Ok(ChainResponse::TransactionInfo(
                    self.service
                        .get_txn_info_by_block_and_index(block_id, txn_idx)?,
                ))
            }
            ChainRequest::GetEventsByTxnInfoId { txn_info_id } => Ok(ChainResponse::Events(
                self.service.get_events_by_txn_info_id(txn_info_id)?,
            )),
            ChainRequest::GetEpochInfo() => {
                Ok(ChainResponse::EpochInfo(self.service.epoch_info()?))
            }
            ChainRequest::GetEpochInfoByNumber(number) => Ok(ChainResponse::EpochInfo(
                self.service.get_epoch_info_by_number(number)?,
            )),
            ChainRequest::GetGlobalTimeByNumber(number) => Ok(ChainResponse::GlobalTime(
                self.service.get_global_time_by_number(number)?,
            )),
        }
    }
}

#[derive(Clone)]
pub struct ChainActorRef {
    pub address: Addr<ChainActor>,
}

impl Into<Addr<ChainActor>> for ChainActorRef {
    fn into(self) -> Addr<ChainActor> {
        self.address
    }
}

impl Into<ChainActorRef> for Addr<ChainActor> {
    fn into(self) -> ChainActorRef {
        ChainActorRef { address: self }
    }
}

#[async_trait::async_trait]
impl ChainAsyncService for ChainActorRef {
    async fn get_header_by_hash(&self, hash: &HashValue) -> Result<Option<BlockHeader>> {
        if let ChainResponse::BlockHeader(header) = self
            .address
            .send(ChainRequest::GetHeaderByHash(*hash))
            .await??
        {
            if let Some(h) = *header {
                return Ok(Some(h));
            }
        }
        Ok(None)
    }

    async fn get_block_by_hash(&self, hash: HashValue) -> Result<Block> {
        if let ChainResponse::OptionBlock(block) = self
            .address
            .send(ChainRequest::GetBlockByHash(hash))
            .await??
        {
            match block {
                Some(b) => Ok(*b),
                None => bail!("get block by hash is none: {:?}", hash),
            }
        } else {
            bail!("get block by hash error.")
        }
    }

    async fn get_block_state_by_hash(&self, hash: &HashValue) -> Result<Option<BlockState>> {
        if let ChainResponse::BlockState(Some(block_state)) = self
            .address
            .send(ChainRequest::GetBlockStateByHash(*hash))
            .await??
        {
            return Ok(Some(*block_state));
        }
        Ok(None)
    }

    async fn get_block_info_by_hash(&self, hash: &HashValue) -> Result<Option<BlockInfo>> {
        debug!("hash: {:?}", hash);
        if let ChainResponse::OptionBlockInfo(block_info) = self
            .address
            .send(ChainRequest::GetBlockInfoByHash(*hash))
            .await??
        {
            return Ok(*block_info);
        }
        Ok(None)
    }

    async fn get_transaction(&self, txn_hash: HashValue) -> Result<Transaction, Error> {
        let response = self
            .address
            .send(ChainRequest::GetTransaction(txn_hash))
            .await
            .map_err(Into::<Error>::into)??;
        if let ChainResponse::Transaction(txn) = response {
            Ok(*txn)
        } else {
            bail!("get transaction error.")
        }
    }

    async fn get_transaction_info(
        &self,
        txn_hash: HashValue,
    ) -> Result<Option<TransactionInfo>, Error> {
        let response = self
            .address
            .send(ChainRequest::GetTransactionInfo(txn_hash))
            .await
            .map_err(Into::<Error>::into)??;
        if let ChainResponse::TransactionInfo(txn_info) = response {
            Ok(txn_info)
        } else {
            bail!("get transaction_info error:{:?}", txn_hash)
        }
    }

    async fn get_block_txn_infos(
        &self,
        block_id: HashValue,
    ) -> Result<Vec<TransactionInfo>, Error> {
        let response = self
            .address
            .send(ChainRequest::GetBlockTransactionInfos(block_id))
            .await
            .map_err(Into::<Error>::into)??;
        if let ChainResponse::TransactionInfos(vec_txn_id) = response {
            Ok(vec_txn_id)
        } else {
            bail!("get block's transaction_info error.")
        }
    }

    async fn get_txn_info_by_block_and_index(
        &self,
        block_id: HashValue,
        idx: u64,
    ) -> Result<Option<TransactionInfo>> {
        let response = self
            .address
            .send(ChainRequest::GetTransactionInfoByBlockAndIndex {
                block_id,
                txn_idx: idx,
            })
            .await
            .map_err(Into::<Error>::into)??;
        if let ChainResponse::TransactionInfo(info) = response {
            Ok(info)
        } else {
            bail!("get txn info by block and idx error.")
        }
    }
    async fn get_events_by_txn_info_id(
        &self,
        txn_info_id: HashValue,
    ) -> Result<Option<Vec<ContractEvent>>> {
        let response = self
            .address
            .send(ChainRequest::GetEventsByTxnInfoId { txn_info_id })
            .await
            .map_err(Into::<Error>::into)??;
        if let ChainResponse::Events(events) = response {
            Ok(events)
        } else {
            bail!("get txn info by block and idx error.")
        }
    }

    async fn master_head_header(&self) -> Result<Option<BlockHeader>> {
        if let Ok(ChainResponse::BlockHeader(header)) =
            self.address.send(ChainRequest::CurrentHeader()).await?
        {
            return Ok(*header);
        }
        Ok(None)
    }

    async fn master_head_block(&self) -> Result<Option<Block>> {
        if let ChainResponse::Block(block) = self.address.send(ChainRequest::HeadBlock()).await?? {
            Ok(Some(*block))
        } else {
            Ok(None)
        }
    }

    async fn master_block_by_number(&self, number: BlockNumber) -> Result<Block> {
        if let ChainResponse::Block(block) = self
            .address
            .send(ChainRequest::GetBlockByNumber(number))
            .await
            .map_err(Into::<Error>::into)??
        {
            Ok(*block)
        } else {
            bail!("Get chain block by number response error.")
        }
    }

    async fn master_block_by_uncle(&self, uncle_id: HashValue) -> Result<Option<Block>> {
        if let ChainResponse::OptionBlock(block) = self
            .address
            .send(ChainRequest::GetBlockByUncle(uncle_id))
            .await??
        {
            match block {
                Some(b) => Ok(Some(*b)),
                None => Ok(None),
            }
        } else {
            bail!("get block by hash error.")
        }
    }

    async fn master_blocks_by_number(
        &self,
        number: Option<BlockNumber>,
        count: u64,
    ) -> Result<Vec<Block>> {
        if let ChainResponse::VecBlock(blocks) = self
            .address
            .send(ChainRequest::GetBlocksByNumber(number, count))
            .await
            .map_err(Into::<Error>::into)??
        {
            Ok(blocks)
        } else {
            bail!("Get chain blocks by number response error.")
        }
    }

    async fn master_block_header_by_number(&self, number: BlockNumber) -> Result<BlockHeader> {
        if let ChainResponse::BlockHeader(header) = self
            .address
            .send(ChainRequest::GetBlockHeaderByNumber(number))
            .await
            .map_err(Into::<Error>::into)??
        {
            if let Some(h) = *header {
                return Ok(h);
            }
        }
        bail!("Get chain block header by number response error.")
    }

    async fn master_startup_info(&self) -> Result<StartupInfo> {
        let response = self
            .address
            .send(ChainRequest::GetStartupInfo())
            .await
            .map_err(Into::<Error>::into)??;
        if let ChainResponse::StartupInfo(startup_info) = response {
            Ok(startup_info)
        } else {
            bail!("Get chain info response error.")
        }
    }

    async fn master_head(&self) -> Result<ChainInfo> {
        let response = self
            .address
            .send(ChainRequest::GetHeadChainInfo())
            .await
            .map_err(Into::<Error>::into)??;
        if let ChainResponse::ChainInfo(chain_info) = response {
            Ok(chain_info)
        } else {
            bail!("get head chain info error.")
        }
    }

    async fn epoch_info(&self) -> Result<EpochInfo> {
        let response = self
            .address
            .send(ChainRequest::GetEpochInfo())
            .await
            .map_err(Into::<Error>::into)??;
        if let ChainResponse::EpochInfo(epoch_info) = response {
            Ok(epoch_info)
        } else {
            bail!("get epoch chain info error.")
        }
    }

    async fn get_epoch_info_by_number(&self, number: BlockNumber) -> Result<EpochInfo> {
        let response = self
            .address
            .send(ChainRequest::GetEpochInfoByNumber(number))
            .await
            .map_err(Into::<Error>::into)??;
        if let ChainResponse::EpochInfo(epoch_info) = response {
            Ok(epoch_info)
        } else {
            bail!("get epoch chain info error.")
        }
    }

    async fn get_global_time_by_number(&self, number: BlockNumber) -> Result<GlobalTimeOnChain> {
        let response = self
            .address
            .send(ChainRequest::GetGlobalTimeByNumber(number))
            .await
            .map_err(Into::<Error>::into)??;
        if let ChainResponse::GlobalTime(global_time) = response {
            Ok(global_time)
        } else {
            bail!("get global time error.")
        }
    }

    async fn create_block_template(
        &self,
        author: AccountAddress,
        auth_key_prefix: Option<Vec<u8>>,
        parent_hash: Option<HashValue>,
        txs: Vec<SignedUserTransaction>,
    ) -> Result<BlockTemplate> {
        let address = self.address.clone();
        if let ChainResponse::BlockTemplate(block_template) = address
            .send(ChainRequest::CreateBlockTemplate(
                author,
                auth_key_prefix,
                parent_hash,
                txs,
            ))
            .await??
        {
            Ok(*block_template)
        } else {
            bail!("create block template error")
        }
    }
}

#[cfg(test)]
mod tests;
