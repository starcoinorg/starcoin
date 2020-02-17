// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod chain;
#[cfg(any(test))]
mod mock_chain;
mod starcoin_chain_state;

use actix::{Actor, Addr, Context};
use anyhow::Result;
use config::NodeConfig;
use crypto::HashValue;
use types::block::Block;

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

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
