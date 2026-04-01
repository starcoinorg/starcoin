// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use jsonrpsee::core::{async_trait, RpcResult};
use network_api::{PeerProvider, ReputationChange, BANNED_THRESHOLD};
use network_p2p_core::RawRpcClient;
use network_p2p_types::network_state::NetworkState;
use network_p2p_types::peer_id::PeerId;
use network_types::peer_info::Multiaddr;
use starcoin_network::NetworkServiceRef;
use starcoin_rpc_api::network_manager::NetworkManagerApiServer;
use starcoin_rpc_api::types::StrView;
use std::str::FromStr;

pub struct NetworkManagerRpcImpl {
    service: NetworkServiceRef,
}

impl NetworkManagerRpcImpl {
    pub fn new(service: NetworkServiceRef) -> Self {
        Self { service }
    }
}

#[async_trait]
impl NetworkManagerApiServer for NetworkManagerRpcImpl {
    async fn state(&self) -> RpcResult<NetworkState> {
        let service = self.service.clone();
        service
            .network_state()
            .await
            .map_err(crate::module::map_jsonrpc_err)
    }

    async fn known_peers(&self) -> RpcResult<Vec<PeerId>> {
        let service = self.service.clone();
        let result = service.known_peers().await;
        Ok(result)
    }

    async fn get_address(&self, peer_id: String) -> RpcResult<Vec<Multiaddr>> {
        let service = self.service.clone();
        let peer_id = PeerId::from_str(peer_id.as_str()).map_err(crate::module::map_jsonrpc_err)?;
        let result = service.get_address(peer_id).await;
        Ok(result)
    }

    async fn add_peer(&self, peer: String) -> RpcResult<()> {
        let service = self.service.clone();
        service
            .add_peer(peer)
            .map_err(crate::module::map_jsonrpc_err)
    }

    async fn call_peer(
        &self,
        peer_id: String,
        rpc_method: String,
        message: StrView<Vec<u8>>,
    ) -> RpcResult<StrView<Vec<u8>>> {
        let service = self.service.clone();
        let peer_id = PeerId::from_str(peer_id.as_str()).map_err(crate::module::map_jsonrpc_err)?;
        let response = service
            .send_raw_request(peer_id, rpc_method.into(), message.0)
            .await
            .map_err(crate::module::map_jsonrpc_err)?;
        Ok(StrView(response))
    }

    async fn set_peer_reputation(&self, peer_id: String, reputation: i32) -> RpcResult<()> {
        let service = self.service.clone();
        let peer_id = PeerId::from_str(peer_id.as_str()).map_err(crate::module::map_jsonrpc_err)?;
        let old_reput = service
            .reputations(BANNED_THRESHOLD)
            .await
            .map_err(crate::module::map_jsonrpc_err)?
            .await
            .map_err(|e| crate::module::map_jsonrpc_err(e.into()))?
            .iter()
            .find(|(p, _)| p == &peer_id)
            .ok_or_else(|| crate::module::map_jsonrpc_err(anyhow::anyhow!("Invalid peer id")))?
            .1;
        let reputation_change = reputation.saturating_sub(old_reput);
        service.report_peer(
            peer_id,
            ReputationChange {
                value: reputation_change,
                reason: "Report peer manual",
            },
        );
        Ok(())
    }

    fn ban_peer(&self, peer_id: String, ban: bool) -> RpcResult<()> {
        let service = self.service.clone();
        let peer_id = PeerId::from_str(peer_id.as_str()).map_err(crate::module::map_jsonrpc_err)?;
        service.ban_peer(peer_id, ban);
        Ok(())
    }
}
