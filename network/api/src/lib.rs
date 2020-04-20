use crate::messages::PeerMessage;
use anyhow::*;
use libp2p::PeerId;
use starcoin_types::system_events::SystemEvents;
use std::time::Duration;

pub mod messages;

use async_trait::async_trait;
use starcoin_types::peer_info::PeerInfo;

#[async_trait]
pub trait NetworkService: Send + Sync + Clone + Sized {
    async fn send_peer_message(&self, peer_id: PeerId, msg: PeerMessage) -> Result<()>;
    async fn broadcast_system_event(&self, event: SystemEvents) -> Result<()>;

    fn identify(&self) -> &PeerId;

    async fn send_request_bytes(
        &self,
        peer_id: PeerId,
        message: Vec<u8>,
        time_out: Duration,
    ) -> Result<Vec<u8>>;

    async fn peer_set(&self) -> Result<Vec<PeerInfo>>;

    async fn best_peer_set(&self) -> Result<Vec<PeerInfo>>;

    async fn get_peer(&self, peer_id: &PeerId) -> Result<Option<PeerInfo>>;

    async fn get_self_peer(&self) -> Result<PeerInfo>;

    async fn best_peer(&self) -> Result<Option<PeerInfo>>;

    async fn get_peer_set_size(&self) -> Result<usize>;
}
