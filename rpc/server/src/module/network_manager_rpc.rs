// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::module::map_err;
use futures::future::TryFutureExt;
use futures::FutureExt;
use jsonrpc_core::Result;
use network_api::{PeerProvider, ReputationChange, BANNED_THRESHOLD};
use network_p2p_types::network_state::NetworkState;
use network_rpc_core::RawRpcClient;
use starcoin_network::NetworkServiceRef;
use starcoin_rpc_api::network_manager::NetworkManagerApi;
use starcoin_rpc_api::types::StrView;
use starcoin_rpc_api::FutureResult;
use starcoin_types::peer_info::{Multiaddr, PeerId};
use std::borrow::Cow;
use std::str::FromStr;

pub struct NetworkManagerRpcImpl {
    service: NetworkServiceRef,
}

impl NetworkManagerRpcImpl {
    pub fn new(service: NetworkServiceRef) -> Self {
        Self { service }
    }
}

impl NetworkManagerApi for NetworkManagerRpcImpl {
    fn state(&self) -> FutureResult<NetworkState> {
        let service = self.service.clone();
        let fut = async move { service.network_state().await }.map_err(map_err);
        Box::pin(fut.boxed())
    }

    fn known_peers(&self) -> FutureResult<Vec<PeerId>> {
        let service = self.service.clone();
        let fut = async move {
            let result = service.known_peers().await;
            Ok(result)
        }
        .map_err(map_err);
        Box::pin(fut.boxed())
    }

    fn get_address(&self, peer_id: String) -> FutureResult<Vec<Multiaddr>> {
        let service = self.service.clone();
        let fut = async move {
            let peer_id = PeerId::from_str(peer_id.as_str())?;
            let result = service.get_address(peer_id).await;
            Ok(result)
        }
        .map_err(map_err);
        Box::pin(fut.boxed())
    }

    fn add_peer(&self, peer: String) -> FutureResult<()> {
        let service = self.service.clone();
        let fut = async move { service.add_peer(peer) }.map_err(map_err);
        Box::pin(fut.boxed())
    }

    fn call_peer(
        &self,
        peer_id: String,
        rpc_method: Cow<'static, str>,
        message: StrView<Vec<u8>>,
    ) -> FutureResult<StrView<Vec<u8>>> {
        let service = self.service.clone();
        let fut = async move {
            let peer_id = PeerId::from_str(peer_id.as_str())?;
            let response = service
                .send_raw_request(peer_id, rpc_method, message.0)
                .await?;
            Ok(StrView(response))
        }
        .map_err(map_err);
        Box::pin(fut.boxed())
    }

    fn set_peer_reputation(&self, peer_id: String, reputation: i32) -> FutureResult<()> {
        let service = self.service.clone();
        let fut = async move {
            let peer_id = PeerId::from_str(peer_id.as_str())?;
            let old_reput = service
                .reputations(BANNED_THRESHOLD)
                .await?
                .await?
                .iter()
                .find(|(p, _)| p == &peer_id)
                .ok_or_else(|| anyhow::anyhow!("Invalid peer id"))?
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
        .map_err(map_err);
        Box::pin(fut.boxed())
    }

    fn ban_peer(&self, peer_id: String, ban: bool) -> Result<()> {
        let service = self.service.clone();
        let peer_id = PeerId::from_str(peer_id.as_str()).map_err(map_err)?;
        service.ban_peer(peer_id, ban);
        Ok(())
    }
}
