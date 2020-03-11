use crate::download::Downloader;
use crate::download_body::{DownloadBodyActor, SyncBodyEvent};
use crate::{do_duration, DELAY_TIME};
use actix::prelude::*;
use anyhow::{Error, Result};
use crypto::hash::HashValue;
use network::{
    sync_messages::{
        BatchBodyMsg, BatchHashByNumberMsg, BatchHeaderMsg, BlockBody, DataType, DownloadMessage,
        GetDataByHashMsg, GetHashByNumberMsg, HashWithNumber, LatestStateMsg, ProcessMessage,
    },
    NetworkAsyncService, RPCMessage, RPCRequest, RPCResponse,
};
use std::sync::Arc;
use txpool::TxPoolRef;
use types::peer_info::PeerInfo;

#[derive(Default, Debug, Message)]
#[rtype(result = "Result<()>")]
struct SyncHeaderEvent {
    pub hashs: Vec<HashValue>,
    pub peers: Vec<PeerInfo>,
}

#[derive(Clone)]
pub struct DownloadHeaderActor {
    downloader: Arc<Downloader>,
    peer_info: Arc<PeerInfo>,
    network: NetworkAsyncService<TxPoolRef>,
    download_body: Addr<DownloadBodyActor>,
}

impl DownloadHeaderActor {
    pub fn launch(
        downloader: Arc<Downloader>,
        peer_info: Arc<PeerInfo>,
        network: NetworkAsyncService<TxPoolRef>,
        download_body: Addr<DownloadBodyActor>,
    ) -> Result<Addr<DownloadHeaderActor>> {
        Ok(Actor::create(move |ctx| DownloadHeaderActor {
            downloader,
            peer_info,
            network,
            download_body,
        }))
    }
}

impl Actor for DownloadHeaderActor {
    type Context = Context<Self>;
}

impl Handler<SyncHeaderEvent> for DownloadHeaderActor {
    type Result = Result<()>;
    fn handle(&mut self, event: SyncHeaderEvent, ctx: &mut Self::Context) -> Self::Result {
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
                if let RPCResponse::BatchHeaderAndBodyMsg(_, headers, bodies) = network
                    .clone()
                    .send_request(
                        peer.id.clone().into(),
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
