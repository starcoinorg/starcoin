use crate::download::DownloadActor;
use crate::txn_sync::TxnSyncActor;
use actix::{prelude::*, Actor, Addr, Context, Handler};
use anyhow::Result;
use bus::{BusActor, Subscription};
use config::NodeConfig;
use logger::prelude::*;
use network::PeerEvent;
use network_api::NetworkService;
use starcoin_chain_service::ChainReaderService;
use starcoin_service_registry::ServiceRef;
use starcoin_storage::Store;
use starcoin_sync_api::{PeerNewBlock, SyncNotify};
use starcoin_types::{peer_info::PeerId, startup_info::StartupInfo};
use std::sync::Arc;
use txpool::TxPoolService;

pub struct SyncActor {
    download_address: Addr<DownloadActor>,
    #[allow(dead_code)]
    txn_sync_address: Addr<TxnSyncActor>,
    bus: Addr<BusActor>,
}

impl SyncActor {
    pub fn launch<N>(
        node_config: Arc<NodeConfig>,
        bus: Addr<BusActor>,
        peer_id: Arc<PeerId>,
        chain: ServiceRef<ChainReaderService>,
        txpool: TxPoolService,
        network: N,
        storage: Arc<dyn Store>,
        startup_info: StartupInfo,
    ) -> Result<Addr<SyncActor>>
    where
        N: NetworkService + 'static,
    {
        let txn_sync_addr = TxnSyncActor::launch(txpool.clone(), network.clone(), bus.clone());
        let download_address = DownloadActor::launch(
            node_config,
            peer_id,
            chain,
            network,
            bus.clone(),
            storage.clone(),
            txpool,
            startup_info,
        )?;

        let actor = SyncActor {
            download_address,
            txn_sync_address: txn_sync_addr,
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

        let sync_recipient = ctx.address().recipient::<PeerNewBlock>();
        self.bus
            .send(Subscription {
                recipient: sync_recipient,
            })
            .into_actor(self)
            .then(|_res, act, _ctx| async {}.into_actor(act))
            .wait(ctx);
        info!("SyncActor started");
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!("SyncActor stopped");
    }
}

impl Handler<PeerNewBlock> for SyncActor {
    type Result = ();
    fn handle(&mut self, msg: PeerNewBlock, ctx: &mut Self::Context) -> Self::Result {
        let new_block = SyncNotify::NewHeadBlock(msg.get_peer_id(), Box::new(msg.get_block()));
        self.download_address
            .send(new_block)
            .into_actor(self)
            .then(|_result, act, _ctx| async {}.into_actor(act))
            .wait(ctx);
    }
}

impl Handler<PeerEvent> for SyncActor {
    type Result = Result<()>;

    fn handle(&mut self, msg: PeerEvent, ctx: &mut Self::Context) -> Self::Result {
        match msg {
            PeerEvent::Open(open_peer_id, _) => {
                debug!("connect new peer:{:?}", open_peer_id);
                let download_msg = SyncNotify::NewPeerMsg(open_peer_id);
                self.download_address
                    .send(download_msg)
                    .into_actor(self)
                    .then(|_result, act, _ctx| async {}.into_actor(act))
                    .wait(ctx);
            }
            PeerEvent::Close(close_peer_id) => {
                debug!("disconnect peer: {:?}", close_peer_id);
                let download_msg = SyncNotify::ClosePeerMsg(close_peer_id);
                self.download_address
                    .send(download_msg)
                    .into_actor(self)
                    .then(|_result, act, _ctx| async {}.into_actor(act))
                    .wait(ctx);
            }
        }

        Ok(())
    }
}
