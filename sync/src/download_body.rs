use crate::download::Downloader;
use crate::{do_duration, DELAY_TIME};
use actix::prelude::*;
use anyhow::{Error, Result};
use network::{
    sync_messages::{
        BatchBodyMsg, BatchHashByNumberMsg, BatchHeaderMsg, BlockBody, DataType, DownloadMessage,
        GetDataByHashMsg, GetHashByNumberMsg, HashWithNumber, LatestStateMsg, ProcessMessage,
    },
    NetworkAsyncService, RPCMessage, RPCRequest, RPCResponse,
};
use std::sync::Arc;
use txpool::TxPoolRef;
use types::{block::BlockHeader, peer_info::PeerInfo};

#[derive(Default, Debug, Message)]
#[rtype(result = "Result<()>")]
pub struct SyncBodyEvent {
    pub headers: Vec<BlockHeader>,
    pub peers: Vec<PeerInfo>,
}

#[derive(Clone)]
pub struct DownloadBodyActor {
    downloader: Arc<Downloader>,
    peer_info: Arc<PeerInfo>,
    network: NetworkAsyncService<TxPoolRef>,
}

impl DownloadBodyActor {
    pub fn launch(
        downloader: Arc<Downloader>,
        peer_info: Arc<PeerInfo>,
        network: NetworkAsyncService<TxPoolRef>,
    ) -> Result<Addr<DownloadBodyActor>> {
        Ok(Actor::create(move |ctx| DownloadBodyActor {
            downloader,
            peer_info,
            network,
        }))
    }
}

impl Actor for DownloadBodyActor {
    type Context = Context<Self>;
}

impl Handler<SyncBodyEvent> for DownloadBodyActor {
    type Result = Result<()>;
    fn handle(&mut self, event: SyncBodyEvent, ctx: &mut Self::Context) -> Self::Result {
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
                if let RPCResponse::BatchHeaderAndBodyMsg(_, _, bodies) = network
                    .clone()
                    .send_request(
                        peer.id.clone().into(),
                        get_data_by_hash_req.clone(),
                        do_duration(DELAY_TIME),
                    )
                    .await
                    .unwrap()
                {
                    Downloader::do_blocks(downloader, headers, bodies.bodies).await;
                    break;
                };
            }
        });

        Ok(())
    }
}
