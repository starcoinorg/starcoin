use crate::pool::TTLPool;
use crate::proto::{BlockBody, HashWithHeight};
use crate::Synchronizer;
use actix::{Actor, Addr, Context};
use anyhow::Result;
use chain::ChainActor;
use config::NodeConfig;
use network::NetworkActor;
use std::sync::Arc;
use types::block::BlockHeader;

pub struct SyncActor {
    block_sync: BlockSync,
}

impl SyncActor {
    pub fn launch(
        _node_config: &NodeConfig,
        _network: Addr<NetworkActor>,
        chain: Addr<ChainActor>,
    ) -> Result<Addr<SyncActor>> {
        let block_sync = BlockSync {
            chain,
            hash_pool: TTLPool::new(),
            header_pool: TTLPool::new(),
            body_pool: TTLPool::new(),
        };
        let actor = SyncActor { block_sync };
        Ok(actor.start())
    }
}

impl Actor for SyncActor {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        println!("Sync actor started");
    }
}

pub struct BlockSync {
    chain: Addr<ChainActor>,
    hash_pool: TTLPool<HashWithHeight>,
    header_pool: TTLPool<BlockHeader>,
    body_pool: TTLPool<BlockBody>,
}

impl Synchronizer for BlockSync {}
