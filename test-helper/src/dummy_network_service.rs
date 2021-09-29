use futures::channel::oneshot::Receiver;
use futures::future::BoxFuture;
use futures::FutureExt;
use network_api::messages::NotificationMessage;
use network_api::{messages::PeerMessage, NetworkService, PeerId, PeerProvider, ReputationChange};
use starcoin_logger::prelude::*;
use starcoin_types::peer_info::PeerInfo;

#[derive(Clone)]
pub struct DummyNetworkService {
    self_info: PeerInfo,
}

impl DummyNetworkService {
    pub fn new(self_info: PeerInfo) -> Self {
        Self { self_info }
    }
}

impl Default for DummyNetworkService {
    fn default() -> Self {
        Self::new(PeerInfo::random())
    }
}

#[async_trait::async_trait]
impl NetworkService for DummyNetworkService {
    fn send_peer_message(&self, _msg: PeerMessage) {}

    fn broadcast(&self, _notification: NotificationMessage) {}
}

impl PeerProvider for DummyNetworkService {
    fn peer_set(&self) -> BoxFuture<anyhow::Result<Vec<PeerInfo>>> {
        async { Ok(vec![]) }.boxed()
    }

    fn get_peer(&self, _peer_id: PeerId) -> BoxFuture<anyhow::Result<Option<PeerInfo>>> {
        async { Ok(None) }.boxed()
    }

    fn get_self_peer(&self) -> BoxFuture<anyhow::Result<PeerInfo>> {
        let info = self.self_info.clone();
        async { Ok(info) }.boxed()
    }

    fn report_peer(&self, peer_id: PeerId, cost_benefit: ReputationChange) {
        info!("report_peer {:?}: reputation: {:?}", peer_id, cost_benefit);
    }

    fn reputations(
        &self,
        _reputation_threshold: i32,
    ) -> BoxFuture<'_, anyhow::Result<Receiver<Vec<(PeerId, i32)>>>> {
        unimplemented!()
    }

    fn ban_peer(&self, _peer_id: PeerId, _ban: bool) {
        unimplemented!()
    }
}
