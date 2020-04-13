use crate::download::Downloader;
use crate::download_body::{DownloadBodyActor, SyncBodyEvent};
use crate::helper::send_sync_request;
use actix::prelude::*;
use anyhow::Result;
use crypto::hash::HashValue;
use network::NetworkAsyncService;
use network_p2p_api::sync_messages::{DataType, GetDataByHashMsg, ProcessMessage};
use network_p2p_api::sync_messages::{SyncRpcRequest, SyncRpcResponse};
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
        let get_data_by_hash_msg = GetDataByHashMsg {
            hashs: event.hashs.clone(),
            data_type: DataType::HEADER,
        };

        let get_data_by_hash_req = SyncRpcRequest::GetDataByHashMsg(
            ProcessMessage::GetDataByHashMsg(get_data_by_hash_msg),
        );

        let network = self.network.clone();
        let peers = event.peers.clone();
        let download_body = self.download_body.clone();
        Arbiter::spawn(async move {
            for peer in peers.clone() {
                if let SyncRpcResponse::BatchHeaderAndBodyMsg(headers, _bodies, _infos) =
                    send_sync_request(&network, peer.get_peer_id(), get_data_by_hash_req.clone())
                        .await
                        .unwrap()
                {
                    download_body.do_send(SyncBodyEvent {
                        headers: headers.headers,
                        peers: peers.clone(),
                    });
                    break;
                };
            }
        });

        Ok(())
    }
}
