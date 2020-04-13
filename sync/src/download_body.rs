use crate::download::Downloader;
use crate::helper::get_block_by_hash;
use actix::prelude::*;
use anyhow::Result;
use crypto::hash::HashValue;
use executor::TransactionExecutor;
use logger::prelude::*;
use network::NetworkAsyncService;
use std::sync::Arc;
use traits::Consensus;
use types::{block::BlockHeader, peer_info::PeerInfo};

#[derive(Default, Debug, Message)]
#[rtype(result = "Result<()>")]
pub struct SyncBodyEvent {
    pub headers: Vec<BlockHeader>,
    pub peers: Vec<PeerInfo>,
}

#[derive(Clone)]
pub struct DownloadBodyActor<E, C>
where
    E: TransactionExecutor + Sync + Send + 'static + Clone,
    C: Consensus + Sync + Send + 'static + Clone,
{
    downloader: Arc<Downloader<E, C>>,
    peer_info: Arc<PeerInfo>,
    network: NetworkAsyncService,
}

impl<E, C> DownloadBodyActor<E, C>
where
    E: TransactionExecutor + Sync + Send + 'static + Clone,
    C: Consensus + Sync + Send + 'static + Clone,
{
    pub fn _launch(
        downloader: Arc<Downloader<E, C>>,
        peer_info: Arc<PeerInfo>,
        network: NetworkAsyncService,
    ) -> Result<Addr<DownloadBodyActor<E, C>>> {
        Ok(Actor::create(move |_ctx| DownloadBodyActor {
            downloader,
            peer_info,
            network,
        }))
    }
}

impl<E, C> Actor for DownloadBodyActor<E, C>
where
    E: TransactionExecutor + Sync + Send + 'static + Clone,
    C: Consensus + Sync + Send + 'static + Clone,
{
    type Context = Context<Self>;
}

impl<E, C> Handler<SyncBodyEvent> for DownloadBodyActor<E, C>
where
    E: TransactionExecutor + Sync + Send + 'static + Clone,
    C: Consensus + Sync + Send + 'static + Clone,
{
    type Result = Result<()>;
    fn handle(&mut self, event: SyncBodyEvent, _ctx: &mut Self::Context) -> Self::Result {
        let hashs: Vec<HashValue> = event.headers.iter().map(|h| h.id().clone()).collect();

        let network = self.network.clone();
        let peers = event.peers.clone();
        let downloader = self.downloader.clone();

        let headers = event.headers;
        Arbiter::spawn(async move {
            for peer in peers {
                match get_block_by_hash(&network, peer.get_peer_id().clone(), hashs.clone()).await {
                    Ok((_, bodies, infos)) => {
                        Downloader::do_blocks(
                            downloader,
                            headers.clone(),
                            bodies.bodies,
                            infos.infos,
                        )
                        .await;
                        break;
                    }
                    Err(e) => {
                        error!("error: {:?}", e);
                    }
                };
            }
        });

        Ok(())
    }
}
