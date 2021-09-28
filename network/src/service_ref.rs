// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::service::NetworkActorService;
use crate::worker::RPC_PROTOCOL_PREFIX;
use crate::PeerMessage;
use anyhow::{format_err, Result};
use futures::channel::oneshot::Receiver;
use futures::future::BoxFuture;
use futures::FutureExt;
use log::warn;
use network_api::messages::NotificationMessage;
use network_api::{NetworkService, PeerProvider, ReputationChange, SupportedRpcProtocol};
use network_p2p_types::network_state::NetworkState;
use network_p2p_types::{IfDisconnected, Multiaddr, RequestFailure};
use network_rpc_core::{NetRpcError, RawRpcClient};
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

    fn reputations(
        &self,
        reputation_threshold: i32,
    ) -> BoxFuture<'_, Result<Receiver<Vec<(PeerId, i32)>>>> {
        self.service_ref.reputations(reputation_threshold)
    }

    fn ban_peer(&self, peer_id: PeerId, ban: bool) {
        self.service_ref.ban_peer(peer_id, ban)
    }
}

impl SupportedRpcProtocol for NetworkServiceRef {
    fn is_supported(&self, peer_id: PeerId, protocol: Cow<'static, str>) -> BoxFuture<bool> {
        async move {
            if let Ok(Some(peer_info)) = self.get_peer(peer_id).await {
                return peer_info.is_support_rpc_protocol(protocol);
            }
            false
        }
        .boxed()
    }
}

impl RawRpcClient for NetworkServiceRef {
    fn send_raw_request(
        &self,
        peer_id: PeerId,
        rpc_path: Cow<'static, str>,
        message: Vec<u8>,
    ) -> BoxFuture<Result<Vec<u8>>> {
        async move {
            if self.get_peer(peer_id.clone()).await?.is_none() {
                return Err(RequestFailure::NotConnected.into());
            }
            let protocol = format!("{}{}", RPC_PROTOCOL_PREFIX, rpc_path);
            if self
                .is_supported(peer_id.clone(), protocol.clone().into())
                .await
            {
                self.network_service
                    .request(
                        peer_id.into(),
                        protocol,
                        message,
                        IfDisconnected::ImmediateError,
                    )
                    .await
                    .map_err(|e| e.into())
            } else {
                warn!(
                    "[network] remote peer: {:?} not support rpc protocol :{:?}",
                    peer_id, protocol
                );
                Err(NetRpcError::method_not_fount(rpc_path)).map_err(|e| e.into())
            }
        }
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

    pub async fn is_connected(&self, peer_id: PeerId) -> bool {
        self.network_service.is_connected(peer_id.into()).await
    }
}
