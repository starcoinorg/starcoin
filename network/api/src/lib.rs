use crate::messages::PeerMessage;
use anyhow::*;
use libp2p::PeerId;
use starcoin_types::system_events::SystemEvents;
use std::time::Duration;

pub mod messages;

use async_trait::async_trait;
use starcoin_types::peer_info::PeerInfo;

#[async_trait]
pub trait NetworkService: Send + Sync + Clone + Sized + std::marker::Unpin {
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

#[derive(Clone)]
pub struct DummyNetworkService {
    peer_id: PeerId,
    peers: Vec<PeerInfo>,
}

impl DummyNetworkService {
    pub fn new(peer_id: PeerId, peers: Vec<PeerInfo>) -> Self {
        Self { peer_id, peers }
    }
}

#[async_trait]
impl NetworkService for DummyNetworkService {
    async fn send_peer_message(&self, _peer_id: PeerId, _msg: PeerMessage) -> Result<()> {
        Ok(())
    }

    async fn broadcast_system_event(&self, _event: SystemEvents) -> Result<()> {
        Ok(())
    }

    fn identify(&self) -> &PeerId {
        &self.peer_id
    }

    async fn send_request_bytes(
        &self,
        _peer_id: PeerId,
        _message: Vec<u8>,
        _time_out: Duration,
    ) -> Result<Vec<u8>> {
        Ok(vec![])
    }

    async fn peer_set(&self) -> Result<Vec<PeerInfo>> {
        Ok(self.peers.clone())
    }

    async fn best_peer_set(&self) -> Result<Vec<PeerInfo>> {
        Ok(self.peers.clone())
    }

    async fn get_peer(&self, _peer_id: &PeerId) -> Result<Option<PeerInfo>> {
        Ok(None)
    }

    async fn get_self_peer(&self) -> Result<PeerInfo> {
        Ok(self.peers.get(0).expect("should have").clone())
    }

    async fn best_peer(&self) -> Result<Option<PeerInfo>> {
        Ok(Some(self.peers.get(0).expect("should have").clone()))
    }

    async fn get_peer_set_size(&self) -> Result<usize> {
        Ok(0)
    }
}
