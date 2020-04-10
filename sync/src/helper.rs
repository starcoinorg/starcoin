use crate::{do_duration, DELAY_TIME};
use anyhow::Result;
use network::NetworkAsyncService;
use network_p2p_api::sync_messages::{SyncRpcRequest, SyncRpcResponse};
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
