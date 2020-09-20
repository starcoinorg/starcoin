use crate::helper;
use actix::prelude::*;
use anyhow::{bail, Result};
use bus::{Bus, BusActor};
use logger::prelude::*;
use network_api::NetworkService;
use starcoin_network_rpc_api::{gen_client::NetworkRpcClient, GetTxns};
use starcoin_sync_api::StartSyncTxnEvent;
use starcoin_txpool_api::TxPoolSyncService;
use txpool::TxPoolService;
use types::peer_info::PeerId;

#[derive(Clone)]
pub struct TxnSyncActor<N>
where
    N: NetworkService + 'static,
{
    bus: Addr<BusActor>,
    inner: Inner<N>,
}

impl<N> TxnSyncActor<N>
where
    N: NetworkService + 'static,
{
    pub fn launch(txpool: TxPoolService, network: N, bus: Addr<BusActor>) -> Addr<TxnSyncActor<N>> {
        let actor = TxnSyncActor {
            inner: Inner {
                pool: txpool,
                rpc_client: NetworkRpcClient::new(network.clone()),
                network_service: network,
            },
            bus,
        };
        actor.start()
    }
}

impl<N> actix::Actor for TxnSyncActor<N>
where
    N: NetworkService + 'static,
{
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

impl<N> actix::Handler<StartSyncTxnEvent> for TxnSyncActor<N>
where
    N: NetworkService + 'static,
{
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
struct Inner<N>
where
    N: NetworkService + 'static,
{
    pool: TxPoolService,
    rpc_client: NetworkRpcClient,
    network_service: N,
}

impl<N> Inner<N>
where
    N: NetworkService + 'static,
{
    async fn sync_txn(self) -> Result<()> {
        // get all peers and sort by difficulty, try peer with max difficulty.
        let best_peers = self.network_service.peer_selector().await?.top(10);
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
