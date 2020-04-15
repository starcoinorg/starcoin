use crate::get_txns_handler::GetTxnsHandler;
use crate::helper;
use actix::prelude::*;
use anyhow::Result;
use bus::{Bus, BusActor, Subscription};
use crypto::hash::{CryptoHash, HashValue};
use futures::FutureExt;
use network::{NetworkAsyncService, PeerMessage};
use rand::RngCore;
use starcoin_sync_api::sync_messages::{GetTxns, StartSyncTxnEvent};
use starcoin_txpool_api::TxPoolAsyncService;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use txpool::{TxPoolRef, TxStatus};
use types::peer_info::{PeerId, PeerInfo};
use types::transaction::SignedUserTransaction;

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
            .then(|res, act, ctx| match res {
                Err(e) => {
                    error!("fail to subscribe start_sync_txn event, err: {:?}", e);
                    ctx.terminate();
                }
                Ok(_) => {}
            })
            .wait(ctx);

        info!("Network actor started ",);
    }
}
impl actix::Handler<StartSyncTxnEvent> for TxnSyncActor {
    type Result = ();

    fn handle(
        &mut self,
        msg: StartSyncTxnEvent,
        ctx: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        self.inner
            .clone()
            .sync_txn()
            .into_actor(self)
            .map(|res, act, ctx| {
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
            let txn_data = helper::get_txns(&network, best_peer.peer_id, GetTxns)
                .await?
                .txns;
            let import_result = pool.add_txns(txn_data).await?;
            let succ_num = import_result.iter().filter(|r| r.is_ok()).collect();
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
