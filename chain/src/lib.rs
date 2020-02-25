// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod chain;

pub use chain::BlockChain;

pub mod chain_service;
pub mod chain_state_store;
pub mod mem_chain;
pub mod message;

use crate::chain_service::ChainServiceImpl;
use crate::message::ChainResponse;
use actix::dev::ToEnvelope;
use actix::fut::wrap_future;
use actix::prelude::*;
use anyhow::{Error, Result};
use config::NodeConfig;
use consensus::dummy::DummyConsensus;
use crypto::{hash::CryptoHash, HashValue};
use executor::mock_executor::MockExecutor;
use futures::compat::Future01CompatExt;
use futures_locks::RwLock;
use logger::prelude::*;
use message::ChainRequest;
use std::sync::Arc;
use storage::StarcoinStorage;
pub use tests::gen_mem_chain_for_test;
use traits::{AsyncChain, ChainAsyncService, ChainReader, ChainService, ChainWriter};
use types::block::{Block, BlockHeader, BlockNumber, BlockTemplate};

/// actor for block chain.
pub struct ChainActor {
    //TODO use Generic Parameter for Executor and Consensus.
    service: ChainServiceImpl<MockExecutor, DummyConsensus>,
}

impl ChainActor {
    pub fn launch(
        config: Arc<NodeConfig>,
        storage: Arc<StarcoinStorage>,
    ) -> Result<Addr<ChainActor>> {
        let actor = ChainActor {
            service: ChainServiceImpl::new(config, storage)?,
        };
        Ok(actor.start())
    }
}

impl Actor for ChainActor {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        info!("ChainActor actor started");
    }
}

impl Handler<ChainRequest> for ChainActor {
    type Result = Result<ChainResponse>;

    fn handle(&mut self, msg: ChainRequest, ctx: &mut Self::Context) -> Self::Result {
        match msg {
            ChainRequest::CreateBlock(times) => {
                let head_block = self.service.head_block();
                let mut parent_block_hash = head_block.crypto_hash();
                for i in 0..times {
                    info!("parent_block_hash: {:?}", parent_block_hash);
                    let current_block_header =
                        BlockHeader::new_block_header_for_test(parent_block_hash, i);
                    let current_block = Block::new_nil_block_for_test(current_block_header);
                    parent_block_hash = current_block.crypto_hash();
                    self.service.try_connect(current_block)?;
                }
                Ok(ChainResponse::None)
            }
            ChainRequest::CurrentHeader() => {
                Ok(ChainResponse::BlockHeader(self.service.current_header()))
            }
            ChainRequest::GetHeaderByHash(hash) => Ok(ChainResponse::BlockHeader(
                self.service.get_header(hash).unwrap().unwrap(),
            )),
            ChainRequest::HeadBlock() => Ok(ChainResponse::Block(self.service.head_block())),
            ChainRequest::GetHeaderByNumber(number) => Ok(ChainResponse::BlockHeader(
                self.service.get_header_by_number(number)?.unwrap(),
            )),
            ChainRequest::GetBlockByNumber(number) => Ok(ChainResponse::Block(
                self.service.get_block_by_number(number)?.unwrap(),
            )),
            ChainRequest::CreateBlockTemplate() => Ok(ChainResponse::BlockTemplate(
                //TODO get txn from txpool.
                self.service.create_block_template(vec![]).unwrap(),
            )),
            ChainRequest::GetBlockByHash(hash) => Ok(ChainResponse::OptionBlock(
                self.service.get_block(hash).unwrap(),
            )),
            ChainRequest::ConnectBlock(block) => {
                self.service.try_connect(block).unwrap();
                Ok(ChainResponse::None)
            }
        }
    }
}

pub struct ChainActorRef<A>
where
    A: Actor + Handler<ChainRequest>,
    A::Context: ToEnvelope<A, ChainRequest>,
    A: Send,
{
    pub address: Addr<A>,
}

impl<A> Clone for ChainActorRef<A>
where
    A: Actor + Handler<ChainRequest>,
    A::Context: ToEnvelope<A, ChainRequest>,
    A: Send,
{
    fn clone(&self) -> ChainActorRef<A> {
        ChainActorRef {
            address: self.address.clone(),
        }
    }
}

impl<A> Into<Addr<A>> for ChainActorRef<A>
where
    A: Actor + Handler<ChainRequest>,
    A::Context: ToEnvelope<A, ChainRequest>,
    A: Send,
{
    fn into(self) -> Addr<A> {
        self.address
    }
}

impl<A> Into<ChainActorRef<A>> for Addr<A>
where
    A: Actor + Handler<ChainRequest>,
    A::Context: ToEnvelope<A, ChainRequest>,
    A: Send,
{
    fn into(self) -> ChainActorRef<A> {
        ChainActorRef { address: self }
    }
}

#[async_trait::async_trait]
impl<A> AsyncChain for ChainActorRef<A>
where
    A: Actor + Handler<ChainRequest>,
    A::Context: ToEnvelope<A, ChainRequest>,
    A: Send,
{
    async fn current_header(self) -> Option<BlockHeader> {
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

    async fn head_block(self) -> Option<Block> {
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

    async fn get_header_by_number(self, number: BlockNumber) -> Option<BlockHeader> {
        if let ChainResponse::BlockHeader(header) = self
            .address
            .send(ChainRequest::GetHeaderByNumber(number))
            .await
            .unwrap()
            .unwrap()
        {
            Some(header)
        } else {
            None
        }
    }

    async fn get_block_by_number(self, number: BlockNumber) -> Option<Block> {
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

    async fn create_block_template(self) -> Option<BlockTemplate> {
        let address = self.address.clone();
        drop(self);
        if let ChainResponse::BlockTemplate(block_template) = address
            .send(ChainRequest::CreateBlockTemplate())
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
        info!("hash: {:?}", hash);
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
}

#[async_trait::async_trait]
impl<A> ChainAsyncService for ChainActorRef<A>
where
    A: Actor + Handler<ChainRequest>,
    A::Context: ToEnvelope<A, ChainRequest>,
    A: Send,
{
    async fn try_connect(self, block: Block) -> Result<()> {
        self.address
            .send(ChainRequest::ConnectBlock(block))
            .await
            .map_err(|e| Into::<Error>::into(e))?;
        Ok(())
    }
}

mod tests;
