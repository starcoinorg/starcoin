use crate::{do_duration, DELAY_TIME};
use anyhow::{format_err, Result};
use network::NetworkAsyncService;
use network_p2p_api::sync_messages::{
    BatchHashByNumberMsg, GetHashByNumberMsg, ProcessMessage, SyncRpcRequest, SyncRpcResponse,
};
use starcoin_canonical_serialization::SCSCodec;
use types::peer_info::PeerId;

pub async fn send_sync_request(
    network: &NetworkAsyncService,
    peer_id: PeerId,
    req: SyncRpcRequest,
) -> Result<SyncRpcResponse> {
    let request = req.encode()?;
    let response = network
        .send_request_bytes(peer_id.into(), request, do_duration(DELAY_TIME))
        .await?;
    SyncRpcResponse::decode(&response)
}

pub async fn get_hash_by_number(
    network: &NetworkAsyncService,
    peer_id: PeerId,
    req: GetHashByNumberMsg,
) -> Result<BatchHashByNumberMsg> {
    let request = SyncRpcRequest::GetHashByNumberMsg(ProcessMessage::GetHashByNumberMsg(req));

    if let SyncRpcResponse::BatchHashByNumberMsg(batch_hash_by_number_msg) =
        send_sync_request(&network, peer_id, request).await?
    {
        Ok(batch_hash_by_number_msg)
    } else {
        Err(format_err!("{:?}", "error SyncRpcResponse type"))
    }
}
