use crate::download::Downloader;
use crate::download_body::{DownloadBodyActor, SyncBodyEvent};
use crate::helper::get_header_by_hash;
use actix::prelude::*;
use anyhow::Result;
use crypto::hash::HashValue;
use executor::TransactionExecutor;
use logger::prelude::*;
use network::NetworkAsyncService;
use std::sync::Arc;
use traits::Consensus;
use types::peer_info::PeerInfo;

#[derive(Default, Debug, Message)]
#[rtype(result = "Result<()>")]
struct SyncHeaderEvent {
    pub hashs: Vec<HashValue>,
    pub peers: Vec<PeerInfo>,
}

#[derive(Clone)]
pub struct DownloadHeaderActor<E, C>
where
    E: TransactionExecutor + Sync + Send + 'static + Clone,
    C: Consensus + Sync + Send + 'static + Clone,
{
    downloader: Arc<Downloader<E, C>>,
    peer_info: Arc<PeerInfo>,
    network: NetworkAsyncService,
    download_body: Addr<DownloadBodyActor<E, C>>,
}

impl<E, C> DownloadHeaderActor<E, C>
where
    E: TransactionExecutor + Sync + Send + 'static + Clone,
    C: Consensus + Sync + Send + 'static + Clone,
{
    pub fn _launch(
        downloader: Arc<Downloader<E, C>>,
        peer_info: Arc<PeerInfo>,
        network: NetworkAsyncService,
        download_body: Addr<DownloadBodyActor<E, C>>,
    ) -> Result<Addr<DownloadHeaderActor<E, C>>> {
        Ok(Actor::create(move |_ctx| DownloadHeaderActor {
            downloader,
            peer_info,
            network,
            download_body,
        }))
    }
}

impl<E, C> Actor for DownloadHeaderActor<E, C>
where
    E: TransactionExecutor + Sync + Send + 'static + Clone,
    C: Consensus + Sync + Send + 'static + Clone,
{
    type Context = Context<Self>;
}

impl<E, C> Handler<SyncHeaderEvent> for DownloadHeaderActor<E, C>
where
    E: TransactionExecutor + Sync + Send + 'static + Clone,
    C: Consensus + Sync + Send + 'static + Clone,
{
    type Result = Result<()>;
    fn handle(&mut self, event: SyncHeaderEvent, _ctx: &mut Self::Context) -> Self::Result {
        let network = self.network.clone();
        let peers = event.peers.clone();
        let hashs = event.hashs.clone();
        let download_body = self.download_body.clone();
        Arbiter::spawn(async move {
            for peer in peers.clone() {
                match get_header_by_hash(&network, peer.get_peer_id(), hashs.clone()).await {
                    Ok(headers) => {
                        download_body.do_send(SyncBodyEvent {
                            headers: headers.headers,
                            peers: peers.clone(),
                        });
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
