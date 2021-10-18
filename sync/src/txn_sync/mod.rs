use anyhow::{bail, Result};
use logger::prelude::*;
use network::NetworkServiceRef;
use network_api::{NetworkService, PeerProvider, PeerSelector, PeerStrategy};
use starcoin_network_rpc_api::{gen_client::NetworkRpcClient, GetTxnsWithSize, RawRpcClient};
use starcoin_service_registry::{ActorService, EventHandler, ServiceContext};
use starcoin_txpool_api::TxPoolSyncService;
use starcoin_types::peer_info::{PeerId, RpcInfo};
use starcoin_types::system_events::SyncStatusChangeEvent;
use std::sync::Arc;
use txpool::TxPoolService;

#[derive(Clone, Default)]
pub struct TxnSyncService {}

impl ActorService for TxnSyncService {
    fn started(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.unsubscribe::<SyncStatusChangeEvent>();
        Ok(())
    }

    fn stopped(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.unsubscribe::<SyncStatusChangeEvent>();
        Ok(())
    }
}

impl TxnSyncService {
    pub fn sync_txn(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        let network = ctx.get_shared::<NetworkServiceRef>()?;
        let txpool = ctx.get_shared::<TxPoolService>()?;
        let inner = Inner::new(txpool, network);

        // sync txn after block sync task done.
        // because txn verify dependency the latest chain state, such as timestamp on chain.
        // only sync txn if txpool is empty currently.
        if inner.pool.status().txn_count == 0 {
            ctx.wait(async move {
                if let Err(e) = inner.sync_txn().await {
                    error!("handle sync txn event fail: {:?}", e);
                }
            });
        }
        Ok(())
    }
}

impl EventHandler<Self, SyncStatusChangeEvent> for TxnSyncService {
    fn handle_event(&mut self, msg: SyncStatusChangeEvent, ctx: &mut ServiceContext<Self>) {
        let sync_status = msg.0;
        if sync_status.is_synced() {
            if let Err(e) = self.sync_txn(ctx) {
                error!("handle sync txn event fail: {:?}", e);
            }
        }
    }
}

#[derive(Clone)]
struct Inner {
    pool: TxPoolService,
    rpc_client: NetworkRpcClient,
    peer_provider: Arc<dyn PeerProvider>,
}

impl Inner {
    pub fn new<N>(txpool: TxPoolService, network: N) -> Self
    where
        N: NetworkService + RawRpcClient + 'static,
    {
        Self {
            pool: txpool,
            rpc_client: NetworkRpcClient::new(network.clone()),
            peer_provider: Arc::new(network),
        }
    }
    async fn sync_txn(self) -> Result<()> {
        // get all peers and sort by difficulty, try peer with max difficulty.
        let peers = self.peer_provider.peer_set().await?;
        let peer_selector = PeerSelector::new(peers, PeerStrategy::default(), None);
        peer_selector.retain_rpc_peers_by_protocol(
            vec![format!("{}/{}", RpcInfo::RPC_PROTOCOL_PREFIX, "get_txns_from_pool").into()]
                .as_slice(),
        );
        let best_peers = peer_selector.top(10);
        if best_peers.is_empty() {
            info!("No peer to sync txn.");
            return Ok(());
        }
        for peer_id in best_peers {
            match self.sync_txn_from_peer(peer_id).await {
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
        let txn_data = self
            .rpc_client
            .get_txns_from_pool(peer_id.clone(), GetTxnsWithSize { max_size: 100 })
            .await?;
        let import_result = self.pool.add_txns(txn_data);
        let succ_num = import_result.iter().filter(|r| r.is_ok()).count();
        info!("succ to sync {} txn from peer {}", succ_num, peer_id);
        Ok(())
    }
}
