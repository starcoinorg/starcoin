use crate::download::DownloadActor;
use crate::process::ProcessActor;
use crate::txn_sync::TxnSyncActor;
use actix::{prelude::*, Actor, Addr, Context, Handler};
use anyhow::Result;
use bus::{BusActor, Subscription};
use chain::ChainActorRef;
use config::NodeConfig;
use logger::prelude::*;
use network::NetworkAsyncService;
use network::PeerEvent;
use starcoin_state_tree::StateNodeStore;
use starcoin_sync_api::sync_messages::{PeerNewBlock, SyncNotify};
use starcoin_sync_api::SyncMetadata;
use std::sync::Arc;
use traits::Consensus;
use txpool::TxPoolRef;
use types::peer_info::PeerId;

pub struct SyncActor<C>
where
    C: Consensus + Sync + Send + 'static + Clone,
{
    _process_address: Addr<ProcessActor<C>>,
    download_address: Addr<DownloadActor<C>>,
    #[allow(dead_code)]
    txn_sync_address: Addr<TxnSyncActor>,
    bus: Addr<BusActor>,
}

impl<C> SyncActor<C>
where
    C: Consensus + Sync + Send + 'static + Clone,
{
    pub fn launch(
        node_config: Arc<NodeConfig>,
        bus: Addr<BusActor>,
        peer_id: Arc<PeerId>,
        chain: ChainActorRef<C>,
        txpool: TxPoolRef,
        network: NetworkAsyncService,
        state_node_storage: Arc<dyn StateNodeStore>,
        sync_metadata: SyncMetadata,
    ) -> Result<Addr<SyncActor<C>>> {
        let txn_sync_addr = TxnSyncActor::launch(txpool.clone(), network.clone(), bus.clone());
        let process_address = ProcessActor::launch(
            chain.clone(),
            txpool,
            bus.clone(),
            state_node_storage.clone(),
        )?;
        let download_address = DownloadActor::launch(
            node_config,
            peer_id,
            chain,
            network.clone(),
            bus.clone(),
            state_node_storage.clone(),
            sync_metadata.clone(),
        )?;

        let actor = SyncActor {
            download_address,
            _process_address: process_address,
            txn_sync_address: txn_sync_addr,
            bus,
        };
        Ok(actor.start())
    }
}

impl<C> Actor for SyncActor<C>
where
    C: Consensus + Sync + Send + 'static + Clone,
{
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
        info!("Sync actor started");
    }
}

impl<C> Handler<PeerNewBlock> for SyncActor<C>
where
    C: Consensus + Sync + Send + 'static + Clone,
{
    type Result = ();

    fn handle(&mut self, msg: PeerNewBlock, ctx: &mut Self::Context) -> Self::Result {
        let new_block = SyncNotify::NewHeadBlock(msg.get_peer_id(), msg.get_block());
        self.download_address
            .send(new_block)
            .into_actor(self)
            .then(|_result, act, _ctx| async {}.into_actor(act))
            .wait(ctx);
        ()
    }
}

impl<C> Handler<PeerEvent> for SyncActor<C>
where
    C: Consensus + Sync + Send + 'static + Clone,
{
    type Result = Result<()>;

    fn handle(&mut self, msg: PeerEvent, ctx: &mut Self::Context) -> Self::Result {
        match msg {
            PeerEvent::Open(open_peer_id, _) => {
                info!("connect new peer:{:?}", open_peer_id);
                let download_msg = SyncNotify::NewPeerMsg(open_peer_id);
                self.download_address
                    .send(download_msg)
                    .into_actor(self)
                    .then(|_result, act, _ctx| async {}.into_actor(act))
                    .wait(ctx);
            }
            PeerEvent::Close(close_peer_id) => {
                info!("disconnect peer: {:?}", close_peer_id);
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
