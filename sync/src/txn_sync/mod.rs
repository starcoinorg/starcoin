use crate::helper;
use actix::prelude::*;
use anyhow::{bail, Result};
use bus::{Bus, BusActor};
use logger::prelude::*;
use network_api::{NetworkService, PeerProvider};
use starcoin_network_rpc_api::{gen_client::NetworkRpcClient, GetTxns};
use starcoin_sync_api::StartSyncTxnEvent;
use starcoin_txpool_api::TxPoolSyncService;
use starcoin_types::peer_info::PeerId;
use std::sync::Arc;
use txpool::TxPoolService;

#[derive(Clone)]
pub struct TxnSyncActor {
    bus: Addr<BusActor>,
    inner: Inner,
}

impl TxnSyncActor {
    pub fn launch<N>(txpool: TxPoolService, network: N, bus: Addr<BusActor>) -> Addr<TxnSyncActor>
    where
        N: NetworkService + 'static,
    {
        let actor = TxnSyncActor {
            inner: Inner {
                pool: txpool,
                rpc_client: NetworkRpcClient::new(network.clone()),
                peer_provider: Arc::new(network),
            },
            bus,
        };
        actor.start()
    }
}

impl actix::Actor for TxnSyncActor {
    type Context = actix::Context<Self>;

    /// when start, subscribe StartSyncTxnEvent.
    fn started(&mut self, ctx: &mut Self::Context) {
        let myself = ctx.address().recipient::<StartSyncTxnEvent>();
        self.bus
            .clone()
            .subscribe(myself)
            .into_actor(self)
            .map(|res, _act, ctx| {
                if let Err(e) = res {
                    error!("fail to subscribe start_sync_txn event, err: {:?}", e);
                    ctx.terminate();
                }
            })
            .wait(ctx);

        info!("txn sync actor started");
    }
}

impl actix::Handler<StartSyncTxnEvent> for TxnSyncActor {
    type Result = ();

    fn handle(
        &mut self,
        _msg: StartSyncTxnEvent,
        ctx: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        self.inner
            .clone()
            .sync_txn()
            .into_actor(self)
            .map(|res, _act, _ctx| {
                if let Err(e) = res {
                    error!("handle sync txn event fail: {:?}", e);
                }
            })
            .spawn(ctx);
    }
}

#[derive(Clone)]
struct Inner {
    pool: TxPoolService,
    rpc_client: NetworkRpcClient,
    peer_provider: Arc<dyn PeerProvider>,
}

impl Inner {
    async fn sync_txn(self) -> Result<()> {
        // get all peers and sort by difficulty, try peer with max difficulty.
        let best_peers = self.peer_provider.peer_selector().await?.top(10);
        if best_peers.is_empty() {
            info!("No peer to sync txn.");
            return Ok(());
        }
        for peer in best_peers {
            match self.sync_txn_from_peer(peer.peer_id).await {
                Ok(_) => {
                    return Ok(());
                }
                Err(e) => {
                    error!("fail to sync txn from peer, e: {:?}", e);
                }
            }
        }

        bail!("fail to sync txn from all peers")
    }
    async fn sync_txn_from_peer(&self, peer_id: PeerId) -> Result<()> {
        let txn_data = helper::get_txns(&self.rpc_client, peer_id.clone(), GetTxns { ids: None })
            .await?
            .txns;
        let import_result = self.pool.add_txns(txn_data);
        let succ_num = import_result.iter().filter(|r| r.is_ok()).count();
        info!("succ to sync {} txn from peer {}", succ_num, peer_id);
        Ok(())
    }
}
