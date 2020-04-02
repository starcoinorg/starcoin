use crate::download::Downloader;
use crate::download_body::{DownloadBodyActor, SyncBodyEvent};
use crate::{do_duration, DELAY_TIME};
use actix::prelude::*;
use anyhow::Result;
use consensus::Consensus;
use crypto::hash::HashValue;
use executor::TransactionExecutor;
use network::{NetworkAsyncService, RPCRequest, RPCResponse};
use network_p2p_api::sync_messages::{DataType, GetDataByHashMsg, ProcessMessage};
use std::sync::Arc;
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
        let get_data_by_hash_msg = GetDataByHashMsg {
            hashs: event.hashs.clone(),
            data_type: DataType::HEADER,
        };

        let get_data_by_hash_req =
            RPCRequest::GetDataByHashMsg(ProcessMessage::GetDataByHashMsg(get_data_by_hash_msg));

        let network = self.network.clone();
        let peers = event.peers.clone();
        let download_body = self.download_body.clone();
        Arbiter::spawn(async move {
            for peer in peers.clone() {
                if let RPCResponse::BatchHeaderAndBodyMsg(headers, _bodies) = network
                    .clone()
                    .send_request(
                        peer.get_peer_id().clone().into(),
                        get_data_by_hash_req.clone(),
                        do_duration(DELAY_TIME),
                    )
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
