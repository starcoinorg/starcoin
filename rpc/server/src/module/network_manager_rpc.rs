// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::module::map_err;
use futures::future::TryFutureExt;
use futures::FutureExt;
use network_p2p_types::network_state::NetworkState;
use starcoin_network::NetworkServiceRef;
use starcoin_rpc_api::network_manager::NetworkManagerApi;
use starcoin_rpc_api::FutureResult;
use starcoin_types::peer_info::{Multiaddr, PeerId};
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
}
