// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::messages::{NotificationMessage, PeerMessage};
use anyhow::*;
use async_trait::async_trait;
use network_rpc_core::RawRpcClient;

pub mod messages;
mod peer_message_handler;
mod peer_provider;

pub use network_p2p_types::Multiaddr;
pub use network_p2p_types::MultiaddrWithPeerId;
pub use network_p2p_types::ReputationChange;
pub use peer_message_handler::PeerMessageHandler;
pub use peer_provider::{PeerProvider, PeerSelector};
pub use starcoin_types::peer_info::{PeerId, PeerInfo};

#[async_trait]
pub trait NetworkService:
    Send + Sync + Clone + Sized + std::marker::Unpin + RawRpcClient + PeerProvider
{
    /// send notification message to peer.
    async fn send_peer_message(&self, msg: PeerMessage) -> Result<()>;
    /// Broadcast notification to all connected peers
    async fn broadcast(&self, notification: NotificationMessage);

    fn report_peer(&self, peer_id: PeerId, cost_benefit: ReputationChange);
}
