use crate::download::DownloadService;
use actix::{prelude::*, Actor, Addr, Context, Handler};
use anyhow::Result;
use bus::{BusActor, Subscription};
use logger::prelude::*;
use network::PeerEvent;
use starcoin_service_registry::{
    ActorService, EventHandler, ServiceContext, ServiceFactory, ServiceRef,
};
use starcoin_sync_api::{PeerNewBlock, SyncNotify};
use starcoin_types::{peer_info::PeerId, startup_info::StartupInfo};

//TODO should remove this Service?
pub struct SyncService {
    download_service: ServiceRef<DownloadService>,
}

impl SyncService {
    pub fn new(download_service: ServiceRef<DownloadService>) -> Self {
        Self { download_service }
    }
}

impl ServiceFactory<Self> for SyncService {
    fn create(ctx: &mut ServiceContext<SyncService>) -> Result<SyncService> {
        Ok(Self::new(ctx.service_ref::<DownloadService>()?.clone()))
    }
}

impl ActorService for SyncService {
    fn started(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.subscribe::<PeerEvent>();
        ctx.subscribe::<PeerNewBlock>();
        Ok(())
    }

    fn stopped(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.unsubscribe::<PeerEvent>();
        ctx.unsubscribe::<PeerNewBlock>();
        Ok(())
    }
}

impl EventHandler<Self, PeerEvent> for SyncService {
    fn handle_event(&mut self, msg: PeerEvent, ctx: &mut ServiceContext<SyncService>) {
        if let Err(e) = match msg {
            PeerEvent::Open(open_peer_id, _) => {
                debug!("connect new peer:{:?}", open_peer_id);
                let download_msg = SyncNotify::NewPeerMsg(open_peer_id);
                self.download_service.notify(download_msg)
            }
            PeerEvent::Close(close_peer_id) => {
                debug!("disconnect peer: {:?}", close_peer_id);
                let download_msg = SyncNotify::ClosePeerMsg(close_peer_id);
                self.download_service.notify(download_msg)
            }
        } {
            error!("Notify to download error {:?}", e);
        }
    }
}

impl EventHandler<Self, PeerNewBlock> for SyncService {
    fn handle_event(&mut self, msg: PeerNewBlock, ctx: &mut ServiceContext<SyncService>) {
        let new_block = SyncNotify::NewHeadBlock(msg.get_peer_id(), Box::new(msg.get_block()));
        if let Err(e) = self.download_service.notify(new_block) {
            error!("Notify to download error: {:?}", e);
        }
    }
}
