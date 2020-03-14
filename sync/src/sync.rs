use crate::download::DownloadActor;
use crate::process::ProcessActor;
use actix::{prelude::*, Actor, Addr, Context, Handler};
use anyhow::Result;
use bus::{BusActor, Subscription};
use network::sync_messages::{ProcessMessage, SyncMessage};
use network::PeerEvent;
use types::peer_info::PeerInfo;

pub struct SyncActor {
    process_address: Addr<ProcessActor>,
    download_address: Addr<DownloadActor>,
    bus: Addr<BusActor>,
}

impl SyncActor {
    pub fn launch(
        // _node_config: &NodeConfig,
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
        let peer_recipient = ctx.address().recipient::<PeerEvent>();
        self.bus
            .send(Subscription {
                recipient: peer_recipient,
            })
            .into_actor(self)
            .then(|_res, act, _ctx| async {}.into_actor(act))
            .wait(ctx);

        let sync_recipient = ctx.address().recipient::<SyncMessage>();
        self.bus
            .send(Subscription {
                recipient: sync_recipient,
            })
            .into_actor(self)
            .then(|_res, act, _ctx| async {}.into_actor(act))
            .wait(ctx);
        info!("Sync actor started");
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

impl Handler<PeerEvent> for SyncActor {
    type Result = Result<()>;

    fn handle(&mut self, msg: PeerEvent, ctx: &mut Self::Context) -> Self::Result {
        match msg {
            PeerEvent::Open(open_peer) => {
                info!("connect new peer:{:?}", open_peer);
                let peer_info = PeerInfo::new(open_peer);
                let process_msg = ProcessMessage::NewPeerMsg(peer_info);
                self.process_address
                    .send(process_msg)
                    .into_actor(self)
                    .then(|_result, act, _ctx| async {}.into_actor(act))
                    .wait(ctx);
            }
            _ => {}
        }

        Ok(())
    }
}
