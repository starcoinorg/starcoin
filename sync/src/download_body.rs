use crate::download::Downloader;
use crate::{do_duration, DELAY_TIME};
use actix::prelude::*;
use anyhow::Result;
use executor::TransactionExecutor;
use network::{NetworkAsyncService, RPCRequest, RPCResponse};
use network_p2p_api::sync_messages::{DataType, GetDataByHashMsg, ProcessMessage};
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
        let hashs = event.headers.iter().map(|h| h.id().clone()).collect();
        let get_data_by_hash_msg = GetDataByHashMsg {
            hashs,
            data_type: DataType::BODY,
        };

        let get_data_by_hash_req =
            RPCRequest::GetDataByHashMsg(ProcessMessage::GetDataByHashMsg(get_data_by_hash_msg));

        let network = self.network.clone();
        let peers = event.peers.clone();
        let downloader = self.downloader.clone();

        let headers = event.headers;
        Arbiter::spawn(async move {
            for peer in peers {
                if let RPCResponse::BatchHeaderAndBodyMsg(_, bodies, infos) = network
                    .clone()
                    .send_request(
                        peer.get_peer_id().clone().into(),
                        get_data_by_hash_req.clone(),
                        do_duration(DELAY_TIME),
                    )
                    .await
                    .unwrap()
                {
                    Downloader::do_blocks(downloader, headers, bodies.bodies, infos.infos).await;
                    break;
                };
            }
        });

        Ok(())
    }
}
