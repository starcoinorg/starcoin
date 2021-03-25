// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::service::NetworkActorService;
use crate::worker::RPC_PROTOCOL_PREFIX;
use crate::PeerMessage;
use anyhow::{format_err, Result};
use futures::future::BoxFuture;
use futures::{FutureExt, TryFutureExt};
use network_api::messages::NotificationMessage;
use network_api::{NetworkService, PeerProvider, ReputationChange};
use network_p2p_types::network_state::NetworkState;
use network_p2p_types::{IfDisconnected, Multiaddr};
use network_rpc_core::RawRpcClient;
use starcoin_service_registry::ServiceRef;
use starcoin_types::peer_info::PeerId;
use starcoin_types::peer_info::PeerInfo;
use std::borrow::Cow;
use std::sync::Arc;

//TODO Service registry should support custom service ref.
#[derive(Clone)]
pub struct NetworkServiceRef {
    //hold a network_p2p's network_service for directly send message to NetworkWorker.
    network_service: Arc<network_p2p::NetworkService>,
    service_ref: ServiceRef<NetworkActorService>,
}

impl NetworkService for NetworkServiceRef {
    fn send_peer_message(&self, msg: PeerMessage) {
        self.service_ref.send_peer_message(msg)
    }

    fn broadcast(&self, notification: NotificationMessage) {
        self.service_ref.broadcast(notification)
    }
}

impl PeerProvider for NetworkServiceRef {
    fn peer_set(&self) -> BoxFuture<Result<Vec<PeerInfo>>> {
        self.service_ref.peer_set()
    }

    fn get_peer(&self, peer_id: PeerId) -> BoxFuture<Result<Option<PeerInfo>>> {
        self.service_ref.get_peer(peer_id)
    }

    fn get_self_peer(&self) -> BoxFuture<'_, Result<PeerInfo>> {
        self.service_ref.get_self_peer()
    }

    fn report_peer(&self, peer_id: PeerId, cost_benefit: ReputationChange) {
        self.service_ref.report_peer(peer_id, cost_benefit)
    }
}

impl RawRpcClient for NetworkServiceRef {
    fn send_raw_request(
        &self,
        peer_id: PeerId,
        rpc_path: Cow<'static, str>,
        message: Vec<u8>,
    ) -> BoxFuture<Result<Vec<u8>>> {
        let protocol = format!("{}{}", RPC_PROTOCOL_PREFIX, rpc_path);
        self.network_service
            .request(
                peer_id.into(),
                protocol,
                message,
                IfDisconnected::ImmediateError,
            )
            .map_err(|e| e.into())
            .boxed()
    }
}

impl NetworkServiceRef {
    pub fn new(
        network_service: Arc<network_p2p::NetworkService>,
        service_ref: ServiceRef<NetworkActorService>,
    ) -> Self {
        Self {
            network_service,
            service_ref,
        }
    }
    pub fn add_peer(&self, peer: String) -> Result<()> {
        self.network_service
            .add_reserved_peer(peer)
            .map_err(|e| format_err!("{:?}", e))
    }

    pub async fn network_state(&self) -> Result<NetworkState> {
        self.network_service
            .network_state()
            .await
            .map_err(|_| format_err!("request cancel."))
    }

    pub async fn known_peers(&self) -> Vec<PeerId> {
        self.network_service
            .known_peers()
            .await
            .into_iter()
            .map(|peer_id| peer_id.into())
            .collect()
    }

    pub async fn get_address(&self, peer_id: PeerId) -> Vec<Multiaddr> {
        self.network_service.get_address(peer_id.into()).await
    }
}
