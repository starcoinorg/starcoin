use crate::helper;
use anyhow::{bail, Result};
use logger::prelude::*;
use network::NetworkAsyncService;
use network_api::{NetworkService, PeerProvider};
use starcoin_network_rpc_api::{gen_client::NetworkRpcClient, GetTxns};
use starcoin_service_registry::{
    ActorService, EventHandler, ServiceContext, ServiceFactory, ServiceRequest,
};
use starcoin_sync_api::StartSyncTxnEvent;
use starcoin_txpool_api::TxPoolSyncService;
use starcoin_types::peer_info::PeerId;
use std::sync::Arc;
use txpool::TxPoolService;

#[derive(Clone)]
pub struct TxnSyncService {
    inner: Inner,
}

impl ActorService for TxnSyncService {
    fn started(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.subscribe::<StartSyncTxnEvent>();
        Ok(())
    }

    fn stopped(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.unsubscribe::<StartSyncTxnEvent>();
        Ok(())
    }
}

impl ServiceFactory<Self> for TxnSyncService {
    fn create(ctx: &mut ServiceContext<TxnSyncService>) -> Result<TxnSyncService> {
        let txpool = ctx.get_shared::<TxPoolService>()?;
        let network = ctx.get_shared::<NetworkAsyncService>()?;
        Ok(Self::new(txpool, network))
    }
}

impl TxnSyncService {
    pub fn new<N>(txpool: TxPoolService, network: N) -> Self
    where
        N: NetworkService + 'static,
    {
        Self {
            inner: Inner {
                pool: txpool,
                rpc_client: NetworkRpcClient::new(network.clone()),
                peer_provider: Arc::new(network),
            },
        }
    }
}

impl EventHandler<Self, StartSyncTxnEvent> for TxnSyncService {
    fn handle_event(&mut self, _msg: StartSyncTxnEvent, ctx: &mut ServiceContext<TxnSyncService>) {
        let inner = self.inner.clone();
        ctx.wait(async move {
            if let Err(e) = inner.sync_txn().await {
                error!("handle sync txn event fail: {:?}", e);
            }
        });
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
