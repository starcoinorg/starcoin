// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
#![deny(clippy::integer_arithmetic)]

use crate::messages::{
    BanPeer, GetPeerById, GetPeerSet, GetSelfPeer, NotificationMessage, PeerMessage,
    PeerReputations, ReportReputation,
};
use anyhow::*;
use futures::future::BoxFuture;
use futures::FutureExt;
use starcoin_logger::prelude::*;
use starcoin_service_registry::{ActorService, EventHandler, ServiceHandler, ServiceRef};
use std::sync::mpsc::TrySendError;

pub mod messages;
mod peer_message_handler;
mod peer_provider;
pub mod peer_score;
#[cfg(test)]
mod tests;

pub use network_p2p_types::Multiaddr;
pub use network_p2p_types::MultiaddrWithPeerId;
pub use network_p2p_types::{ReputationChange, BANNED_THRESHOLD};
pub use peer_message_handler::PeerMessageHandler;
pub use peer_provider::PeerDetail;
pub use peer_provider::{PeerProvider, PeerSelector, PeerStrategy};

use futures::channel::oneshot::Receiver;
pub use starcoin_types::peer_info::{PeerId, PeerInfo};
use std::borrow::Cow;

pub trait NetworkService: Send + Sync + Clone + Sized + std::marker::Unpin + PeerProvider {
    /// send notification message to a peer.
    fn send_peer_message(&self, msg: PeerMessage);
    /// Broadcast notification to all connected peers
    fn broadcast(&self, notification: NotificationMessage);
}

pub trait NetworkActor:
    ActorService
    + EventHandler<Self, PeerMessage>
    + EventHandler<Self, NotificationMessage>
    + EventHandler<Self, ReportReputation>
    + EventHandler<Self, BanPeer>
    + ServiceHandler<Self, GetPeerSet>
    + ServiceHandler<Self, PeerReputations>
    + ServiceHandler<Self, GetSelfPeer>
    + ServiceHandler<Self, GetPeerById>
{
}

impl<S> PeerProvider for ServiceRef<S>
where
    S: NetworkActor,
{
    fn peer_set(&self) -> BoxFuture<'_, Result<Vec<PeerInfo>>> {
        self.send(GetPeerSet).boxed()
    }

    fn get_peer(&self, peer_id: PeerId) -> BoxFuture<'_, Result<Option<PeerInfo>>> {
        self.send(GetPeerById { peer_id }).boxed()
    }

    fn get_self_peer(&self) -> BoxFuture<'_, Result<PeerInfo>> {
        self.send(GetSelfPeer).boxed()
    }

    fn report_peer(&self, peer_id: PeerId, cost_benefit: ReputationChange) {
        if let Err(e) = self.notify(ReportReputation {
            peer_id,
            change: cost_benefit,
        }) {
            debug!("report_peer error: {}.", e);
        }
    }

    fn reputations(
        &self,
        reputation_threshold: i32,
    ) -> BoxFuture<'_, Result<Receiver<Vec<(PeerId, i32)>>>> {
        self.send(PeerReputations {
            threshold: reputation_threshold,
        })
        .boxed()
    }
    fn ban_peer(&self, peer_id: PeerId, ban: bool) {
        if let Err(e) = self.notify(BanPeer { peer_id, ban }) {
            debug!("ban peer error:{}", e)
        }
    }
}

impl<S> NetworkService for ServiceRef<S>
where
    S: NetworkActor,
{
    fn send_peer_message(&self, msg: PeerMessage) {
        if let Err(e) = self.notify(msg) {
            let msg = match &e {
                TrySendError::Full(msg) => msg,
                TrySendError::Disconnected(msg) => msg,
            };
            warn!("Send message to peer {} error: {}.", msg.peer_id, e);
        }
    }

    fn broadcast(&self, notification: NotificationMessage) {
        if let Err(e) = self.notify(notification) {
            warn!("Broadcast network notification error: {}.", e);
        }
    }
}

pub trait SupportedRpcProtocol {
    fn is_supported(&self, peer_id: PeerId, rpc_protocol: Cow<'static, str>) -> BoxFuture<bool>;
}

pub trait BroadcastProtocolFilter {
    fn peer_info(&self, peer_id: &PeerId) -> Option<PeerInfo>;

    fn filter(&self, peer_set: Vec<PeerId>, notif_protocol: Cow<'static, str>) -> Vec<PeerId> {
        peer_set
            .into_iter()
            .filter(|peer_id| {
                if let Some(peer_info) = self.peer_info(peer_id) {
                    if peer_info.is_support_notif_protocol(notif_protocol.clone()) {
                        return true;
                    }
                }
                false
            })
            .collect()
    }

    fn is_supported(&self, peer_id: &PeerId, notif_protocol: Cow<'static, str>) -> bool;
}
