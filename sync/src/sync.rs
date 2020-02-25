use crate::download::{DownloadActor, Downloader};
use crate::message::{BlockBody, DownloadMessage, HashWithNumber, SyncMessage};
use crate::pool::TTLPool;
use crate::process::{ProcessActor, Processor};
use actix::{prelude::*, Actor, Addr, Context, Handler};
use anyhow::Result;
use bus::{Bus, BusActor, Subscription};
use chain::{ChainActor, ChainActorRef};
use config::NodeConfig;
use futures_locks::RwLock;
use network::NetworkActor;
use std::sync::Arc;
use types::{block::BlockHeader, peer_info::PeerInfo, system_events::SystemEvents};

pub struct SyncActor {
    process_address: Addr<ProcessActor>,
    download_address: Addr<DownloadActor>,
    bus: Addr<BusActor>,
}

impl SyncActor {
    pub fn launch(
        // _node_config: &NodeConfig,
        // _network: Addr<NetworkActor>,
        //        chain: Addr<ChainActor>,
        bus: Addr<BusActor>,
        process_address: Addr<ProcessActor>,
        download_address: Addr<DownloadActor>,
    ) -> Result<Addr<SyncActor>> {
        let actor = SyncActor {
            download_address,
            process_address,
            bus,
        };
        Ok(actor.start())
    }
}

impl Actor for SyncActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let recipient = ctx.address().recipient::<SystemEvents>();
        self.bus
            .send(Subscription { recipient })
            .into_actor(self)
            .then(|_res, act, _ctx| async {}.into_actor(act))
            .wait(ctx);
        println!("Sync actor started");
    }
}

impl Handler<SyncMessage> for SyncActor {
    type Result = ();

    fn handle(&mut self, msg: SyncMessage, ctx: &mut Self::Context) -> Self::Result {
        match msg {
            SyncMessage::DownloadMessage(download_msg) => {
                self.download_address
                    .send(download_msg)
                    .into_actor(self)
                    .then(|_result, act, _ctx| async {}.into_actor(act))
                    .wait(ctx);
            }
            SyncMessage::ProcessMessage(process_msg) => {
                self.process_address
                    .send(process_msg)
                    .into_actor(self)
                    .then(|_result, act, _ctx| async {}.into_actor(act))
                    .wait(ctx);
            }
        }
    }
}

impl Handler<SystemEvents> for SyncActor {
    type Result = ();

    fn handle(&mut self, msg: SystemEvents, ctx: &mut Self::Context) -> Self::Result {
        println!("mined block.");
        match msg {
            SystemEvents::MinedBlock(new_block) => {
                self.download_address
                    .send(DownloadMessage::NewBlock(new_block))
                    .into_actor(self)
                    .then(|_result, act, _ctx| async {}.into_actor(act))
                    .wait(ctx);
            }
            _ => {}
        }
    }
}

#[derive(Clone)]
pub struct SyncFlow {
    pub downloader: Arc<RwLock<Downloader>>,
    pub processor: Arc<RwLock<Processor>>,
    pub peer_info: PeerInfo,
}

impl SyncFlow {
    pub fn new(peer_info: PeerInfo, chain_reader: ChainActorRef<ChainActor>) -> Self {
        let downloader = Arc::new(RwLock::new(Downloader::new(chain_reader.clone())));
        let processor = Arc::new(RwLock::new(Processor::new(chain_reader)));
        SyncFlow {
            downloader,
            processor,
            peer_info,
        }
    }
}
