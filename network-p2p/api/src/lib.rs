use crate::messages::{PeerMessage, RPCRequest, RPCResponse};
use anyhow::*;
use libp2p::PeerId;
use starcoin_types::system_events::SystemEvents;
use std::time::Duration;

pub mod messages;
pub mod sync_messages;

use async_trait::async_trait;

#[async_trait]
pub trait Network: Send + Sync {
    async fn send_peer_message(&self, peer_id: PeerId, msg: PeerMessage) -> Result<()>;
    async fn broadcast_system_event(&self, event: SystemEvents) -> Result<()>;
    async fn send_request(
        &self,
        peer_id: PeerId,
        message: RPCRequest,
        time_out: Duration,
    ) -> Result<RPCResponse>;

    fn identify(&self) -> &PeerId;

    async fn send_request_bytes(
        &self,
        peer_id: PeerId,
        message: Vec<u8>,
        time_out: Duration,
    ) -> Result<Vec<u8>>;
}
