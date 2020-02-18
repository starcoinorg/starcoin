// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod chain;
mod mem_chain;
mod starcoin_chain_state;

use actix::prelude::*;
use anyhow::{Error, Result};
use config::NodeConfig;
use consensus::ChainReader;
use crypto::HashValue;
use types::block::{Block, BlockHeader, BlockTemplate};

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

pub trait BlockChain {
    fn get_block_by_hash(&self, hash: HashValue) -> Option<Block>;
    fn try_connect(&mut self, block: Block) -> Result<()>;
}

pub struct ChainActorRef(Addr<ChainActor>);

impl ChainReader for ChainActorRef {
    fn current_header(&self) -> BlockHeader {
        unimplemented!()
    }

    fn get_header(&self, hash: HashValue) -> BlockHeader {
        unimplemented!()
    }

    fn get_header_by_number(&self, number: u64) -> BlockHeader {
        unimplemented!()
    }

    fn get_block(&self, hash: HashValue) -> Block {
        unimplemented!()
    }

    fn create_block_template(&self) -> Result<BlockTemplate, Error> {
        unimplemented!()
    }
}

mod tests;
