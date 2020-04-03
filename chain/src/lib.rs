// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod chain;

pub use chain::BlockChain;

pub mod chain_service;
pub mod message;
pub use chain_service::to_block_chain_collection;
pub use chain_service::BlockChainCollection;

use crate::chain_service::ChainServiceImpl;
use crate::message::ChainResponse;
use actix::prelude::*;
use anyhow::{bail, Error, Result};
use bus::{BusActor, Subscription};
use config::NodeConfig;
use consensus::Consensus;
use crypto::HashValue;
use executor::TransactionExecutor;
use logger::prelude::*;
use message::ChainRequest;
use network::network::NetworkAsyncService;
use std::sync::Arc;
use storage::Storage;
use traits::{ChainAsyncService, ChainService};
use txpool::TxPoolRef;
use types::{
    account_address::AccountAddress,
    block::{Block, BlockHeader, BlockInfo, BlockNumber, BlockTemplate},
    startup_info::StartupInfo,
    system_events::SystemEvents,
    transaction::SignedUserTransaction,
};

/// actor for block chain.
pub struct ChainActor<E, C>
where
    E: TransactionExecutor,
    C: Consensus,
{
    //TODO use Generic Parameter for Executor and Consensus.
    service: ChainServiceImpl<E, C, Storage, TxPoolRef>,
    bus: Addr<BusActor>,
}

impl<E, C> ChainActor<E, C>
where
    E: TransactionExecutor + Sync + Send + 'static + Clone,
    C: Consensus + Sync + Send + 'static + Clone,
{
    pub fn launch(
        config: Arc<NodeConfig>,
        startup_info: StartupInfo,
        storage: Arc<Storage>,
        network: Option<NetworkAsyncService>,
        bus: Addr<BusActor>,
        txpool: TxPoolRef,
    ) -> Result<ChainActorRef<E, C>> {
        let actor = ChainActor {
            service: ChainServiceImpl::new(
                config,
                startup_info,
                storage,
                network,
                txpool,
                bus.clone(),
            )?,
            bus,
        }
        .start();
        Ok(actor.into())
    }
}

impl<E, C> Actor for ChainActor<E, C>
where
    E: TransactionExecutor + Sync + Send + 'static,
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

impl<E, C> Handler<ChainRequest> for ChainActor<E, C>
where
    E: TransactionExecutor + Sync + Send + 'static,
    C: Consensus + Sync + Send + 'static,
{
    type Result = Result<ChainResponse>;

    fn handle(&mut self, msg: ChainRequest, _ctx: &mut Self::Context) -> Self::Result {
        match msg {
            ChainRequest::CurrentHeader() => Ok(ChainResponse::BlockHeader(
                self.service.master_head_header(),
            )),
            ChainRequest::GetHeaderByHash(hash) => Ok(ChainResponse::BlockHeader(
                self.service.get_header_by_hash(hash).unwrap().unwrap(),
            )),
            ChainRequest::HeadBlock() => Ok(ChainResponse::Block(self.service.master_head_block())),
            ChainRequest::GetBlockByNumber(number) => Ok(ChainResponse::Block(
                self.service.master_block_by_number(number)?.unwrap(),
            )),
            ChainRequest::CreateBlockTemplate(author, auth_key_prefix, parent_hash, txs) => {
                Ok(ChainResponse::BlockTemplate(
                    self.service
                        .create_block_template(
                            author,
                            auth_key_prefix,
                            parent_hash,
                            types::U256::zero(),
                            txs,
                        )
                        .unwrap(),
                ))
            }
            ChainRequest::GetBlockByHash(hash) => Ok(ChainResponse::OptionBlock(
                self.service.get_block_by_hash(hash).unwrap(),
            )),
            ChainRequest::GetBlockInfoByHash(hash) => Ok(ChainResponse::OptionBlockInfo(
                self.service.get_block_info_by_hash(hash).unwrap(),
            )),
            ChainRequest::ConnectBlock(block, mut block_info) => {
                if block_info.is_none() {
                    self.service.try_connect(block).unwrap();
                } else {
                    self.service
                        .try_connect_with_block_info(block, block_info.take().unwrap())
                        .unwrap();
                }

                Ok(ChainResponse::None)
            }
            ChainRequest::GetStartupInfo() => Ok(ChainResponse::StartupInfo(
                self.service.master_startup_info(),
            )),
            ChainRequest::GenTx() => {
                self.service.gen_tx().unwrap();
                Ok(ChainResponse::None)
            }
        }
    }
}

impl<E, C> Handler<SystemEvents> for ChainActor<E, C>
where
    E: TransactionExecutor + Sync + Send + 'static,
    C: Consensus + Sync + Send + 'static,
{
    type Result = ();

    fn handle(&mut self, msg: SystemEvents, _ctx: &mut Self::Context) -> Self::Result {
        debug!("try connect mined block.");
        match msg {
            SystemEvents::MinedBlock(new_block) => match self.service.try_connect(new_block) {
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
pub struct ChainActorRef<E, C>
where
    E: TransactionExecutor + Sync + Send + 'static + Clone,
    C: Consensus + Sync + Send + 'static + Clone,
{
    pub address: Addr<ChainActor<E, C>>,
}

impl<E, C> Into<Addr<ChainActor<E, C>>> for ChainActorRef<E, C>
where
    E: TransactionExecutor + Sync + Send + 'static + Clone,
    C: Consensus + Sync + Send + 'static + Clone,
{
    fn into(self) -> Addr<ChainActor<E, C>> {
        self.address
    }
}

impl<E, C> Into<ChainActorRef<E, C>> for Addr<ChainActor<E, C>>
where
    E: TransactionExecutor + Sync + Send + 'static + Clone,
    C: Consensus + Sync + Send + 'static + Clone,
{
    fn into(self) -> ChainActorRef<E, C> {
        ChainActorRef { address: self }
    }
}

#[async_trait::async_trait(? Send)]
impl<E, C> ChainAsyncService for ChainActorRef<E, C>
where
    E: TransactionExecutor + Sync + Send + 'static + Clone,
    C: Consensus + Sync + Send + 'static + Clone,
{
    async fn try_connect(self, block: Block) -> Result<()> {
        self.address
            .send(ChainRequest::ConnectBlock(block, None))
            .await
            .map_err(|e| Into::<Error>::into(e))??;
        Ok(())
    }

    async fn try_connect_with_block_info(
        &mut self,
        block: Block,
        block_info: BlockInfo,
    ) -> Result<()> {
        self.address
            .send(ChainRequest::ConnectBlock(block, Some(block_info)))
            .await
            .map_err(|e| Into::<Error>::into(e))??;
        Ok(())
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

    async fn gen_tx(&self) -> Result<()> {
        self.address
            .send(ChainRequest::GenTx())
            .await
            .map_err(|e| Into::<Error>::into(e))??;
        Ok(())
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

    async fn master_block_by_number(self, number: BlockNumber) -> Option<Block> {
        if let ChainResponse::Block(block) = self
            .address
            .send(ChainRequest::GetBlockByNumber(number))
            .await
            .unwrap()
            .unwrap()
        {
            Some(block)
        } else {
            None
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
            Some(block_template)
        } else {
            None
        }
    }

    async fn get_block_by_hash(self, hash: &HashValue) -> Option<Block> {
        debug!("hash: {:?}", hash);
        if let ChainResponse::OptionBlock(block) = self
            .address
            .send(ChainRequest::GetBlockByHash(hash.clone()))
            .await
            .unwrap()
            .unwrap()
        {
            match block {
                Some(b) => Some(b),
                _ => None,
            }
        } else {
            None
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
}

mod tests;
