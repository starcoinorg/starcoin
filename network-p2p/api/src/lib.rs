use crate::messages::PeerMessage;
use anyhow::*;
use libp2p::PeerId;
use starcoin_types::system_events::SystemEvents;
use std::time::Duration;

pub mod messages;

use async_trait::async_trait;

#[async_trait]
pub trait Network: Send + Sync {
    async fn send_peer_message(&self, peer_id: PeerId, msg: PeerMessage) -> Result<()>;
    async fn broadcast_system_event(&self, event: SystemEvents) -> Result<()>;

    fn identify(&self) -> &PeerId;

    async fn send_request_bytes(
        &self,
        peer_id: PeerId,
        message: Vec<u8>,
        time_out: Duration,
    ) -> Result<Vec<u8>>;
}
