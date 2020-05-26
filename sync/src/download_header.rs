use crate::download::Downloader;
use crate::download_body::{DownloadBodyActor, SyncBodyEvent};
use crate::helper::get_header_by_hash;
use actix::prelude::*;
use anyhow::Result;
use crypto::hash::HashValue;
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
pub struct DownloadHeaderActor<C>
where
    C: Consensus + Sync + Send + 'static + Clone,
{
    downloader: Arc<Downloader<C>>,
    peer_info: Arc<PeerInfo>,
    network: NetworkAsyncService,
    download_body: Addr<DownloadBodyActor<C>>,
}

impl<C> DownloadHeaderActor<C>
where
    C: Consensus + Sync + Send + 'static + Clone,
{
    pub fn _launch(
        downloader: Arc<Downloader<C>>,
        peer_info: Arc<PeerInfo>,
        network: NetworkAsyncService,
        download_body: Addr<DownloadBodyActor<C>>,
    ) -> Result<Addr<DownloadHeaderActor<C>>> {
        Ok(Actor::create(move |_ctx| DownloadHeaderActor {
            downloader,
            peer_info,
            network,
            download_body,
        }))
    }
}

impl<C> Actor for DownloadHeaderActor<C>
where
    C: Consensus + Sync + Send + 'static + Clone,
{
    type Context = Context<Self>;
}

impl<C> Handler<SyncHeaderEvent> for DownloadHeaderActor<C>
where
    C: Consensus + Sync + Send + 'static + Clone,
{
    type Result = Result<()>;
    fn handle(&mut self, event: SyncHeaderEvent, _ctx: &mut Self::Context) -> Self::Result {
        let network = self.network.clone();
        let peers = event.peers.clone();
        let hashs = event.hashs;
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
                        error!("{:?}", e);
                    }
                };
            }
        });

        Ok(())
    }
}
