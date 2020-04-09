use crate::download::DownloadActor;
use crate::process::ProcessActor;
use actix::{prelude::*, Actor, Addr, Context, Handler};
use anyhow::Result;
use bus::{BusActor, Subscription};
use chain::ChainActorRef;
use config::NodeConfig;
use executor::TransactionExecutor;
use logger::prelude::*;
use network::NetworkAsyncService;
use network::PeerEvent;
use network_p2p_api::sync_messages::{DownloadMessage, ProcessMessage, SyncMessage};
use starcoin_state_tree::StateNodeStore;
use starcoin_sync_api::SyncMetadata;
use std::sync::Arc;
use traits::Consensus;
use types::peer_info::PeerId;

pub struct SyncActor<E, C>
where
    E: TransactionExecutor + Sync + Send + 'static + Clone,
    C: Consensus + Sync + Send + 'static + Clone,
{
    process_address: Addr<ProcessActor<E, C>>,
    download_address: Addr<DownloadActor<E, C>>,
    bus: Addr<BusActor>,
}

impl<E, C> SyncActor<E, C>
where
    E: TransactionExecutor + Sync + Send + 'static + Clone,
    C: Consensus + Sync + Send + 'static + Clone,
{
    pub fn launch(
        node_config: Arc<NodeConfig>,
        bus: Addr<BusActor>,
        peer_id: Arc<PeerId>,
        chain: ChainActorRef<E, C>,
        network: NetworkAsyncService,
        state_node_storage: Arc<dyn StateNodeStore>,
        sync_metadata: SyncMetadata,
    ) -> Result<Addr<SyncActor<E, C>>> {
        let process_address = ProcessActor::launch(
            Arc::clone(&peer_id),
            chain.clone(),
            network.clone(),
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
            process_address,
            bus,
        };
        Ok(actor.start())
    }
}

impl<E, C> Actor for SyncActor<E, C>
where
    E: TransactionExecutor + Sync + Send + 'static + Clone,
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

impl<E, C> Handler<SyncMessage> for SyncActor<E, C>
where
    E: TransactionExecutor + Sync + Send + 'static + Clone,
    C: Consensus + Sync + Send + 'static + Clone,
{
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

impl<E, C> Handler<PeerEvent> for SyncActor<E, C>
where
    E: TransactionExecutor + Sync + Send + 'static + Clone,
    C: Consensus + Sync + Send + 'static + Clone,
{
    type Result = Result<()>;

    fn handle(&mut self, msg: PeerEvent, ctx: &mut Self::Context) -> Self::Result {
        match msg {
            PeerEvent::Open(open_peer_id, _) => {
                info!("connect new peer:{:?}", open_peer_id);
                let process_msg = ProcessMessage::NewPeerMsg(open_peer_id);
                self.process_address
                    .send(process_msg)
                    .into_actor(self)
                    .then(|_result, act, _ctx| async {}.into_actor(act))
                    .wait(ctx);
            }
            PeerEvent::Close(close_peer_id) => {
                info!("disconnect peer: {:?}", close_peer_id);
                let download_msg = DownloadMessage::ClosePeerMsg(close_peer_id);
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
