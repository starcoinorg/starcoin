// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod chain;

pub use chain::BlockChain;

pub mod chain_service;
pub mod message;
pub mod mock;

pub use chain_service::to_block_chain_collection;
pub use chain_service::BlockChainCollection;

use crate::chain_service::ChainServiceImpl;
use crate::message::ChainResponse;
use actix::prelude::*;
use anyhow::{bail, format_err, Error, Result};
use bus::{BusActor, Subscription};
use config::NodeConfig;
use crypto::HashValue;
use logger::prelude::*;
use message::ChainRequest;
use network::{get_unix_ts, network::NetworkAsyncService};
use starcoin_sync_api::SyncMetadata;
use std::sync::Arc;
use storage::Storage;
use traits::Consensus;
use traits::{ChainAsyncService, ChainService, ConnectResult};
use txpool::TxPoolRef;
use types::{
    account_address::AccountAddress,
    block::{Block, BlockHeader, BlockInfo, BlockNumber, BlockTemplate},
    startup_info::{ChainInfo, StartupInfo},
    system_events::SystemEvents,
    transaction::SignedUserTransaction,
};

/// actor for block chain.
pub struct ChainActor<C>
where
    C: Consensus,
{
    //TODO use Generic Parameter for Executor and Consensus.
    service: ChainServiceImpl<C, Storage, TxPoolRef>,
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
        txpool: TxPoolRef,
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
        let recipient = ctx.address().recipient::<SystemEvents>();
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
            ChainRequest::CurrentHeader() => Ok(ChainResponse::BlockHeader(
                self.service.master_head_header(),
            )),
            ChainRequest::GetHeaderByHash(hash) => Ok(ChainResponse::BlockHeader(
                self.service.get_header_by_hash(hash)?.unwrap(),
            )),
            ChainRequest::HeadBlock() => Ok(ChainResponse::Block(self.service.master_head_block())),
            ChainRequest::GetBlockByNumber(number) => Ok(ChainResponse::Block(
                self.service.master_block_by_number(number)?.unwrap(),
            )),
            ChainRequest::CreateBlockTemplate(author, auth_key_prefix, parent_hash, txs) => Ok(
                ChainResponse::BlockTemplate(self.service.create_block_template(
                    author,
                    auth_key_prefix,
                    parent_hash,
                    txs,
                )?),
            ),
            ChainRequest::GetBlockByHash(hash) => Ok(ChainResponse::OptionBlock(
                self.service.get_block_by_hash(hash)?,
            )),
            ChainRequest::GetBlockInfoByHash(hash) => Ok(ChainResponse::OptionBlockInfo(
                self.service.get_block_info_by_hash(hash)?,
            )),
            ChainRequest::ConnectBlock(block, mut block_info) => {
                let begin_time = get_unix_ts();
                let conn_state = if block_info.is_none() {
                    self.service.try_connect(block, false)?
                } else {
                    self.service
                        .try_connect_with_block_info(block, block_info.take().unwrap())?
                };

                let end_time = get_unix_ts();
                debug!("connect block used time {:?}", (end_time - begin_time));
                Ok(ChainResponse::Conn(conn_state))
            }
            ChainRequest::GetStartupInfo() => Ok(ChainResponse::StartupInfo(
                self.service.master_startup_info(),
            )),
            ChainRequest::GetHeadChainInfo() => Ok(ChainResponse::ChainInfo(
                self.service.master_startup_info().head,
            )),
            ChainRequest::GetBlocksByNumber(number, count) => Ok(ChainResponse::VecBlock(
                self.service.master_blocks_by_number(number, count)?,
            )),

            ChainRequest::GenTx() => {
                self.service.gen_tx()?;
                Ok(ChainResponse::None)
            }
        }
    }
}

impl<C> Handler<SystemEvents> for ChainActor<C>
where
    C: Consensus + Sync + Send + 'static,
{
    type Result = ();

    fn handle(&mut self, msg: SystemEvents, _ctx: &mut Self::Context) -> Self::Result {
        debug!("try connect mined block.");
        match msg {
            SystemEvents::MinedBlock(new_block) => match self.service.try_connect(new_block, false)
            {
                Ok(_) => debug!("Process mined block success."),
                Err(e) => {
                    warn!("Process mined block fail, error: {:?}", e);
                }
            },
            _ => {}
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
            .send(ChainRequest::ConnectBlock(block, None))
            .await
            .map_err(|e| Into::<Error>::into(e))??
        {
            Ok(conn_result)
        } else {
            Err(format_err!("error ChainResponse type."))
        }
    }

    async fn get_header_by_hash(self, hash: &HashValue) -> Option<BlockHeader> {
        if let ChainResponse::BlockHeader(header) = self
            .address
            .send(ChainRequest::GetHeaderByHash(hash.clone()))
            .await
            .unwrap()
            .unwrap()
        {
            Some(header)
        } else {
            None
        }
    }

    async fn get_block_by_hash(self, hash: HashValue) -> Result<Block> {
        debug!("hash: {:?}", hash);
        if let ChainResponse::OptionBlock(block) = self
            .address
            .send(ChainRequest::GetBlockByHash(hash.clone()))
            .await
            .unwrap()
            .unwrap()
        {
            match block {
                Some(b) => Ok(b),
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
            .send(ChainRequest::ConnectBlock(block, Some(block_info)))
            .await
            .map_err(|e| Into::<Error>::into(e))??
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
            .send(ChainRequest::GetBlockInfoByHash(hash.clone()))
            .await
            .unwrap()
            .unwrap()
        {
            match block_info {
                Some(info) => Some(info),
                _ => None,
            }
        } else {
            None
        }
    }

    async fn master_head_header(self) -> Option<BlockHeader> {
        if let ChainResponse::BlockHeader(header) = self
            .address
            .send(ChainRequest::CurrentHeader())
            .await
            .unwrap()
            .unwrap()
        {
            Some(header)
        } else {
            None
        }
    }

    async fn master_head_block(self) -> Option<Block> {
        if let ChainResponse::Block(block) = self
            .address
            .send(ChainRequest::HeadBlock())
            .await
            .unwrap()
            .unwrap()
        {
            Some(block)
        } else {
            None
        }
    }

    async fn master_block_by_number(self, number: BlockNumber) -> Result<Block> {
        if let ChainResponse::Block(block) = self
            .address
            .send(ChainRequest::GetBlockByNumber(number))
            .await
            .map_err(|e| Into::<Error>::into(e))??
        {
            Ok(block)
        } else {
            bail!("Get chain block by number response error.")
        }
    }

    async fn master_blocks_by_number(self, number: u64, count: u64) -> Result<Vec<Block>> {
        if let ChainResponse::VecBlock(blocks) = self
            .address
            .send(ChainRequest::GetBlocksByNumber(number, count))
            .await
            .map_err(|e| Into::<Error>::into(e))??
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
            .map_err(|e| Into::<Error>::into(e))??;
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
            .map_err(|e| Into::<Error>::into(e))
            .unwrap()
            .unwrap();
        if let ChainResponse::ChainInfo(chain_info) = response {
            Ok(chain_info)
        } else {
            bail!("get head chain info error.")
        }
    }

    async fn gen_tx(&self) -> Result<()> {
        self.address
            .send(ChainRequest::GenTx())
            .await
            .map_err(|e| Into::<Error>::into(e))??;
        Ok(())
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
            Some(block_template)
        } else {
            None
        }
    }
}

mod tests;
