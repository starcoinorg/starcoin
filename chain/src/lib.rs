// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod chain;
pub mod mem_chain;
mod message;
mod starcoin_chain_state;

use actix::prelude::*;
use anyhow::{Error, Result};
use config::NodeConfig;
use consensus::ChainReader;
use crypto::HashValue;
use message::ChainMessage;
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

impl Handler<ChainMessage> for ChainActor {
    type Result = ();

    fn handle(&mut self, msg: ChainMessage, ctx: &mut Self::Context) -> Self::Result {
        unimplemented!()
    }
}

pub trait ChainWriter {
    fn get_block_by_hash(&self, hash: &HashValue) -> Option<Block>;
    fn try_connect(&mut self, block: Block) -> Result<()>;
}

pub struct ChainActorRef<A: Actor + Handler<ChainMessage>> {
    pub address: Addr<A>,
}

impl<A: Actor + Handler<ChainMessage>> Clone for ChainActorRef<A> {
    fn clone(&self) -> ChainActorRef<A> {
        ChainActorRef {
            address: self.address.clone(),
        }
    }
}

impl<A: Actor + Handler<ChainMessage>> Into<Addr<A>> for ChainActorRef<A> {
    fn into(self) -> Addr<A> {
        self.address
    }
}

impl<A: Actor + Handler<ChainMessage>> Into<ChainActorRef<A>> for Addr<A> {
    fn into(self) -> ChainActorRef<A> {
        ChainActorRef { address: self }
    }
}

#[async_trait::async_trait]
impl Chain for ChainActorRef<ChainActor> {
    async fn current_header(self) -> BlockHeader {
        unimplemented!()
    }

    async fn get_header(self, hash: &HashValue) -> BlockHeader {
        unimplemented!()
    }

    async fn get_header_by_number(self, number: BlockNumber) -> BlockHeader {
        unimplemented!()
    }

    async fn create_block_template(self) -> Result<BlockTemplate> {
        unimplemented!()
    }

    async fn get_block_by_hash(self, hash: &HashValue) -> Result<Option<Block>> {
        unimplemented!()
    }

    async fn try_connect(self, block: Block) -> Result<()> {
        unimplemented!()
    }
}

mod tests;

pub use tests::gen_mem_chain_for_test;
