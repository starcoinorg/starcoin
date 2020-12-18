use futures::future::BoxFuture;
use network_api::messages::NotificationMessage;
use network_api::{messages::PeerMessage, NetworkService, PeerId, PeerProvider, ReputationChange};
use starcoin_types::peer_info::PeerInfo;

#[derive(Clone)]
pub struct DummyNetworkService;

#[async_trait::async_trait]
impl NetworkService for DummyNetworkService {
    fn send_peer_message(&self, _msg: PeerMessage) {}

    fn broadcast(&self, _notification: NotificationMessage) {}

    fn report_peer(&self, _peer_id: PeerId, _cost_benefit: ReputationChange) {}
}

impl PeerProvider for DummyNetworkService {
    fn peer_set(&self) -> BoxFuture<anyhow::Result<Vec<PeerInfo>>> {
        unimplemented!()
    }

    fn get_peer(&self, _peer_id: PeerId) -> BoxFuture<anyhow::Result<Option<PeerInfo>>> {
        unimplemented!()
    }

    fn get_self_peer(&self) -> BoxFuture<'_, anyhow::Result<PeerInfo>> {
        unimplemented!()
    }
}
