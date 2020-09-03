use crate::messages::PeerMessage;
use anyhow::*;
pub use libp2p::multiaddr::Multiaddr;
use starcoin_types::system_events::NewHeadBlock;
use std::time::Duration;

pub mod messages;
use async_trait::async_trait;
pub use starcoin_types::peer_info::PeerId;
use starcoin_types::peer_info::{PeerInfo, RpcInfo};
use std::borrow::Cow;

#[async_trait]
pub trait NetworkService: Send + Sync + Clone + Sized + std::marker::Unpin {
    async fn send_peer_message(
        &self,
        protocol_name: Cow<'static, [u8]>,
        peer_id: PeerId,
        msg: PeerMessage,
    ) -> Result<()>;
    async fn broadcast_new_head_block(
        &self,
        protocol_name: Cow<'static, [u8]>,
        event: NewHeadBlock,
    ) -> Result<()>;

    fn identify(&self) -> PeerId;

    async fn send_request_bytes(
        &self,
        protocol_name: Cow<'static, [u8]>,
        peer_id: PeerId,
        rpc_path: String,
        message: Vec<u8>,
        time_out: Duration,
    ) -> Result<Vec<u8>>;

    async fn peer_set(&self) -> Result<Vec<PeerInfo>>;

    async fn best_peer_set(&self) -> Result<Vec<PeerInfo>>;

    async fn get_peer(&self, peer_id: &PeerId) -> Result<Option<PeerInfo>>;

    async fn get_self_peer(&self) -> Result<PeerInfo>;

    async fn best_peer(&self) -> Result<Option<PeerInfo>>;

    async fn get_peer_set_size(&self) -> Result<usize>;

    async fn register_rpc_proto(
        &self,
        proto_name: Cow<'static, [u8]>,
        rpc_info: RpcInfo,
    ) -> Result<()>;
}
