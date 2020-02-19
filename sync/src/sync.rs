use crate::inbound::Processor;
use crate::outbound::Downloader;
use crate::pool::TTLPool;
use crate::proto::{BlockBody, HashWithNumber};
use crate::Synchronizer;
use actix::{Actor, Addr, Context};
use anyhow::Result;
use atomic_refcell::AtomicRefCell;
use chain::{mem_chain::MemChain, ChainActor};
use config::NodeConfig;
use network::NetworkActor;
use std::sync::Arc;
use types::{block::BlockHeader, peer_info::PeerInfo};

pub struct SyncActor {
    block_sync: BlockSync,
}

impl SyncActor {
    pub fn launch(
        _node_config: &NodeConfig,
        //        chain: Addr<ChainActor>,
        chain_reader: Arc<AtomicRefCell<MemChain>>,
    ) -> Result<Addr<SyncActor>> {
        let downloader = Downloader::new(chain_reader.clone());
        let processor = Processor::new(chain_reader);
        let peer_info = PeerInfo::random();
        let block_sync = BlockSync {
            downloader,
            processor,
            my_peer_info: peer_info,
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
    pub downloader: Downloader,
    pub processor: Processor,
    pub my_peer_info: PeerInfo,
}

impl BlockSync {
    pub fn new(my_peer_info: PeerInfo, chain_reader: Arc<AtomicRefCell<MemChain>>) -> Self {
        let downloader = Downloader::new(chain_reader.clone());
        let processor = Processor::new(chain_reader);
        BlockSync {
            downloader,
            processor,
            my_peer_info,
        }
    }
}

impl Synchronizer for BlockSync {}
