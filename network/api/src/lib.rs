// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::messages::{
    GetPeerById, GetPeerSet, GetSelfPeer, NotificationMessage, PeerMessage, ReportReputation,
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
pub use network_p2p_types::ReputationChange;
pub use peer_message_handler::PeerMessageHandler;
pub use peer_provider::PeerDetail;
pub use peer_provider::{PeerProvider, PeerSelector, PeerStrategy};

pub use starcoin_types::peer_info::{PeerId, PeerInfo};

pub trait NetworkService: Send + Sync + Clone + Sized + std::marker::Unpin + PeerProvider {
    /// send notification message to a peer.
    fn send_peer_message(&self, msg: PeerMessage);
    /// Broadcast notification to all connected peers
    fn broadcast(&self, notification: NotificationMessage);

    fn report_peer(&self, peer_id: PeerId, cost_benefit: ReputationChange);
}

pub trait NetworkActor:
    ActorService
    + EventHandler<Self, PeerMessage>
    + EventHandler<Self, NotificationMessage>
    + EventHandler<Self, ReportReputation>
    + ServiceHandler<Self, GetPeerSet>
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

    fn report_peer(&self, peer_id: PeerId, cost_benefit: ReputationChange) {
        if let Err(e) = self.notify(ReportReputation {
            peer_id,
            change: cost_benefit,
        }) {
            debug!("report_peer error: {}.", e);
        }
    }
}
