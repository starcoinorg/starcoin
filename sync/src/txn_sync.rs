use crate::helper;
use actix::prelude::*;
use anyhow::Result;
use bus::{Bus, BusActor};
use logger::prelude::*;
use network::NetworkAsyncService;
use starcoin_sync_api::sync_messages::{GetTxns, StartSyncTxnEvent};
use starcoin_txpool_api::TxPoolAsyncService;
use txpool::TxPoolRef;

#[derive(Clone)]
pub struct TxnSyncActor {
    bus: Addr<BusActor>,
    inner: Inner,
}

impl TxnSyncActor {
    pub fn launch(
        txpool: TxPoolRef,
        network: NetworkAsyncService,
        bus: Addr<BusActor>,
    ) -> Addr<TxnSyncActor> {
        let actor = TxnSyncActor {
            inner: Inner {
                pool: txpool,
                network_service: network,
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

        info!("Network actor started ",);
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
                    error!("fail to sync txn from best peers, e: {:?}", e);
                }
            })
            .spawn(ctx);
    }
}

#[derive(Clone)]
struct Inner {
    pool: TxPoolRef,
    network_service: NetworkAsyncService,
}

impl Inner {
    async fn sync_txn(self) -> Result<()> {
        let Inner {
            pool,
            network_service: network,
        } = self;
        let best_peer = network.best_peer().await?;
        if let Some(best_peer) = best_peer {
            let txn_data = helper::get_txns(&network, best_peer.peer_id.clone(), GetTxns)
                .await?
                .txns;
            let import_result = pool.add_txns(txn_data).await?;
            let succ_num = import_result.iter().filter(|r| r.is_ok()).count();
            info!(
                "succ to sync {} txn from peer {}",
                succ_num, best_peer.peer_id
            );
        } else {
            warn!("no best peer, skip sync txn from peers");
        }
        Ok(())
    }
}
