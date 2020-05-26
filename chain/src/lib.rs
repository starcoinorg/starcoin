// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod chain;

pub use chain::BlockChain;

mod chain_metrics;
pub mod chain_service;
pub mod message;
pub mod mock;
pub mod test_helper;

pub use chain_service::ChainServiceImpl;

use crate::message::ChainResponse;
use actix::prelude::*;
use anyhow::{bail, format_err, Error, Result};
use bus::{BusActor, Subscription};
use config::NodeConfig;
use crypto::HashValue;
use logger::prelude::*;
use message::ChainRequest;
use network::{get_unix_ts, NetworkAsyncService};
use starcoin_sync_api::SyncMetadata;
use std::sync::Arc;
use storage::Storage;
use traits::Consensus;
use traits::{ChainAsyncService, ChainService, ConnectResult};
use txpool::TxPoolService;
use types::{
    account_address::AccountAddress,
    block::{Block, BlockHeader, BlockInfo, BlockNumber, BlockState, BlockTemplate},
    startup_info::{ChainInfo, StartupInfo},
    system_events::MinedBlock,
    transaction::{SignedUserTransaction, TransactionInfo},
};

/// actor for block chain.
pub struct ChainActor<C>
where
    C: Consensus,
{
    service: ChainServiceImpl<C, Storage, TxPoolService>,
    bus: Addr<BusActor>,
}

impl<C> ChainActor<C>
where
    C: Consensus + Sync + Send + 'static + Clone,
{
    pub fn launch(
        config: Arc<NodeConfig>,
        startup_info: StartupInfo,
        storage: Arc<Storage>,
        network: Option<NetworkAsyncService>,
        bus: Addr<BusActor>,
        txpool: TxPoolService,
        sync_metadata: SyncMetadata,
    ) -> Result<ChainActorRef<C>> {
        let actor = ChainActor {
            service: ChainServiceImpl::new(
                config,
                startup_info,
                storage,
                network,
                txpool,
                bus.clone(),
                sync_metadata,
            )?,
            bus,
        }
        .start();
        Ok(actor.into())
    }
}

impl<C> Actor for ChainActor<C>
where
    C: Consensus + Sync + Send + 'static,
{
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let recipient = ctx.address().recipient::<MinedBlock>();
        self.bus
            .send(Subscription { recipient })
            .into_actor(self)
            .then(|_res, act, _ctx| async {}.into_actor(act))
            .wait(ctx);
        info!("ChainActor actor started");
    }
}

impl<C> Handler<ChainRequest> for ChainActor<C>
where
    C: Consensus + Sync + Send + 'static,
{
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
                self.service.master_block_by_number(number)?.unwrap(),
            ))),
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
            ChainRequest::ConnectBlock(block, mut block_info) => {
                let begin_time = get_unix_ts();
                let conn_state = if block_info.is_none() {
                    self.service.try_connect(*block, false)?
                } else {
                    self.service
                        .try_connect_with_block_info(*block, *block_info.take().unwrap())?
                };

                let end_time = get_unix_ts();
                debug!("connect block used time {:?}", (end_time - begin_time));
                Ok(ChainResponse::Conn(conn_state))
            }
            ChainRequest::GetStartupInfo() => Ok(ChainResponse::StartupInfo(
                self.service.master_startup_info(),
            )),
            ChainRequest::GetHeadChainInfo() => Ok(ChainResponse::ChainInfo(ChainInfo::new(
                *self.service.master_startup_info().get_master(),
            ))),
            ChainRequest::GetTransaction(hash) => Ok(ChainResponse::Transaction(
                self.service.get_transaction(hash)?.unwrap(),
            )),
            ChainRequest::GetBlocksByNumber(number, count) => Ok(ChainResponse::VecBlock(
                self.service.master_blocks_by_number(number, count)?,
            )),
            ChainRequest::GetTransactionIdByBlock(block_id) => Ok(
                ChainResponse::VecTransactionInfo(self.service.get_block_txn_ids(block_id)?),
            ),
        }
    }
}

impl<C> Handler<MinedBlock> for ChainActor<C>
where
    C: Consensus + Sync + Send + 'static,
{
    type Result = ();

    fn handle(&mut self, msg: MinedBlock, _ctx: &mut Self::Context) -> Self::Result {
        debug!("try connect mined block.");
        let MinedBlock(new_block) = msg;
        match self.service.try_connect(new_block.as_ref().clone(), false) {
            Ok(_) => debug!("Process mined block success."),
            Err(e) => {
                warn!("Process mined block fail, error: {:?}", e);
            }
        }
    }
}

#[derive(Clone)]
pub struct ChainActorRef<C>
where
    C: Consensus + Sync + Send + 'static + Clone,
{
    pub address: Addr<ChainActor<C>>,
}

impl<C> Into<Addr<ChainActor<C>>> for ChainActorRef<C>
where
    C: Consensus + Sync + Send + 'static + Clone,
{
    fn into(self) -> Addr<ChainActor<C>> {
        self.address
    }
}

impl<C> Into<ChainActorRef<C>> for Addr<ChainActor<C>>
where
    C: Consensus + Sync + Send + 'static + Clone,
{
    fn into(self) -> ChainActorRef<C> {
        ChainActorRef { address: self }
    }
}

#[async_trait::async_trait]
impl<C> ChainAsyncService for ChainActorRef<C>
where
    C: Consensus + Sync + Send + 'static + Clone,
{
    async fn try_connect(self, block: Block) -> Result<ConnectResult<()>> {
        if let ChainResponse::Conn(conn_result) = self
            .address
            .send(ChainRequest::ConnectBlock(Box::new(block), None))
            .await
            .map_err(Into::<Error>::into)??
        {
            Ok(conn_result)
        } else {
            Err(format_err!("error ChainResponse type."))
        }
    }

    async fn get_header_by_hash(self, hash: &HashValue) -> Option<BlockHeader> {
        if let ChainResponse::BlockHeader(header) = self
            .address
            .send(ChainRequest::GetHeaderByHash(*hash))
            .await
            .unwrap()
            .unwrap()
        {
            if let Some(h) = *header {
                return Some(h);
            }
        }
        None
    }

    async fn get_block_state_by_hash(self, hash: &HashValue) -> Result<Option<BlockState>> {
        if let ChainResponse::BlockState(Some(block_state)) = self
            .address
            .send(ChainRequest::GetBlockStateByHash(*hash))
            .await?
            .unwrap()
        {
            return Ok(Some(*block_state));
        }
        Ok(None)
    }

    async fn get_block_by_hash(self, hash: HashValue) -> Result<Block> {
        debug!("hash: {:?}", hash);
        if let ChainResponse::OptionBlock(block) = self
            .address
            .send(ChainRequest::GetBlockByHash(hash))
            .await
            .unwrap()
            .unwrap()
        {
            match block {
                Some(b) => Ok(*b),
                None => bail!("get block by hash is none: {:?}", hash),
            }
        } else {
            bail!("get block by hash error.")
        }
    }

    async fn try_connect_with_block_info(
        &mut self,
        block: Block,
        block_info: BlockInfo,
    ) -> Result<ConnectResult<()>> {
        if let ChainResponse::Conn(conn_result) = self
            .address
            .send(ChainRequest::ConnectBlock(
                Box::new(block),
                Some(Box::new(block_info)),
            ))
            .await
            .map_err(Into::<Error>::into)??
        {
            Ok(conn_result)
        } else {
            Err(format_err!("error ChainResponse type."))
        }
    }

    async fn get_block_info_by_hash(self, hash: &HashValue) -> Option<BlockInfo> {
        debug!("hash: {:?}", hash);
        if let ChainResponse::OptionBlockInfo(block_info) = self
            .address
            .send(ChainRequest::GetBlockInfoByHash(*hash))
            .await
            .unwrap()
            .unwrap()
        {
            return *block_info;
        }
        None
    }

    async fn master_head_header(self) -> Option<BlockHeader> {
        if let Ok(ChainResponse::BlockHeader(header)) = self
            .address
            .send(ChainRequest::CurrentHeader())
            .await
            .unwrap()
        {
            return *header;
        }
        None
    }

    async fn master_head_block(self) -> Option<Block> {
        if let ChainResponse::Block(block) = self
            .address
            .send(ChainRequest::HeadBlock())
            .await
            .unwrap()
            .unwrap()
        {
            Some(*block)
        } else {
            None
        }
    }

    async fn master_block_by_number(self, number: BlockNumber) -> Result<Block> {
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

    async fn master_blocks_by_number(
        self,
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

    async fn master_startup_info(self) -> Result<StartupInfo> {
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

    async fn master_head(self) -> Result<ChainInfo> {
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

    async fn get_transaction(self, txn_id: HashValue) -> Result<TransactionInfo, Error> {
        let response = self
            .address
            .send(ChainRequest::GetTransaction(txn_id))
            .await
            .map_err(Into::<Error>::into)??;
        if let ChainResponse::Transaction(transaction_info) = response {
            Ok(transaction_info)
        } else {
            bail!("get transaction error.")
        }
    }

    async fn get_block_txn(self, block_id: HashValue) -> Result<Vec<TransactionInfo>, Error> {
        let response = self
            .address
            .send(ChainRequest::GetTransactionIdByBlock(block_id))
            .await
            .map_err(Into::<Error>::into)??;
        if let ChainResponse::VecTransactionInfo(vec_txn_id) = response {
            Ok(vec_txn_id)
        } else {
            bail!("get block's transaction ids error.")
        }
    }

    async fn create_block_template(
        self,
        author: AccountAddress,
        auth_key_prefix: Option<Vec<u8>>,
        parent_hash: Option<HashValue>,
        txs: Vec<SignedUserTransaction>,
    ) -> Option<BlockTemplate> {
        let address = self.address.clone();
        drop(self);
        if let ChainResponse::BlockTemplate(block_template) = address
            .send(ChainRequest::CreateBlockTemplate(
                author,
                auth_key_prefix,
                parent_hash,
                txs,
            ))
            .await
            .unwrap()
            .unwrap()
        {
            Some(*block_template)
        } else {
            None
        }
    }
}

mod tests;
