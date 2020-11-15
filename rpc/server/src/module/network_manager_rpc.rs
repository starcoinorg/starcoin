// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::module::map_err;
use futures::future::TryFutureExt;
use futures::FutureExt;
use starcoin_network::NetworkAsyncService;
use starcoin_rpc_api::network_manager::NetworkManagerApi;
use starcoin_rpc_api::FutureResult;
use starcoin_types::peer_info::{Multiaddr, PeerId};
use std::str::FromStr;

pub struct NetworkManagerRpcImpl {
    service: NetworkAsyncService,
}

impl NetworkManagerRpcImpl {
    pub fn new(service: NetworkAsyncService) -> Self {
        Self { service }
    }
}

impl NetworkManagerApi for NetworkManagerRpcImpl {
    fn connected_peers(&self) -> FutureResult<Vec<PeerId>> {
        let service = self.service.clone();
        let fut = async move {
            let result = service.connected_peers().await;
            Ok(result)
        }
        .map_err(map_err);
        Box::new(fut.boxed().compat())
    }

    fn get_address(&self, peer_id: String) -> FutureResult<Vec<Multiaddr>> {
        let service = self.service.clone();
        let fut = async move {
            let peer_id = PeerId::from_str(peer_id.as_str())?;
            let result = service.get_address(peer_id).await;
            Ok(result)
        }
        .map_err(map_err);
        Box::new(fut.boxed().compat())
    }

    fn add_peer(&self, peer: String) -> FutureResult<()> {
        let service = self.service.clone();
        let fut = async move { service.add_peer(peer) }.map_err(map_err);
        Box::new(fut.boxed().compat())
    }
}
