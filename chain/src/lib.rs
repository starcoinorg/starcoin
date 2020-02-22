// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod chain;
pub mod mem_chain;
pub mod message;
mod starcoin_chain_state;

use actix::prelude::*;
use anyhow::{Error, Result};
use config::NodeConfig;
use consensus::ChainReader;
use crypto::HashValue;
use message::ChainRequest;
use traits::Chain;
use types::block::{Block, BlockHeader, BlockNumber, BlockTemplate};

pub struct ChainActor {}

impl ChainActor {
    pub fn launch(_node_config: &NodeConfig) -> Result<Addr<ChainActor>> {
        let actor = ChainActor {};
        Ok(actor.start())
    }
}

impl Actor for ChainActor {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        println!("ChainActor actor started");
    }
}

impl Handler<ChainRequest> for ChainActor {
    type Result = ResponseActFuture<Self, Result<ChainResponse>>;

    fn handle(&mut self, msg: ChainRequest, ctx: &mut Self::Context) -> Self::Result {
        unimplemented!()
    }
}

pub trait ChainWriter {
    fn try_connect(&mut self, block: Block) -> Result<()>;
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
impl<A> Chain for ChainActorRef<A>
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
        if let ChainResponse::BlockTemplate(block_template) = self
            .address
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
        println!("hash: {:?}", hash);
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

    async fn try_connect(self, block: Block) -> Result<()> {
        let a = self
            .address
            .send(ChainRequest::ConnectBlock(block))
            .await
            .unwrap()
            .unwrap();
        Ok(())
    }
}

mod tests;

use crate::message::ChainResponse;
use actix::dev::ToEnvelope;
pub use tests::gen_mem_chain_for_test;
