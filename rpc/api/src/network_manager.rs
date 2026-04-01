// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use crate::types::StrView;
use jsonrpsee::{
    core::{RegisterMethodError, RpcResult},
    proc_macros::rpc,
    Methods,
};
use network_p2p_types::network_state::NetworkState;
use network_p2p_types::peer_id::PeerId;
use network_types::peer_info::Multiaddr;
#[rpc(
    client,
    server,
    namespace = "network_manager",
    namespace_separator = "."
)]
pub trait NetworkManagerApi {
    #[method(name = "state")]
    async fn state(&self) -> RpcResult<NetworkState>;

    #[method(name = "known_peers")]
    async fn known_peers(&self) -> RpcResult<Vec<PeerId>>;

    #[method(name = "get_address")]
    async fn get_address(&self, peer_id: String) -> RpcResult<Vec<Multiaddr>>;

    #[method(name = "add_peer")]
    async fn add_peer(&self, peer: String) -> RpcResult<()>;

    /// Call peer's network rpc method.
    #[method(name = "call")]
    async fn call_peer(
        &self,
        peer_id: String,
        rpc_method: String,
        message: StrView<Vec<u8>>,
    ) -> RpcResult<StrView<Vec<u8>>>;

    /// Set peer reputation
    #[method(name = "set_peer_reput")]
    async fn set_peer_reputation(&self, peer_id: String, reputation: i32) -> RpcResult<()>;

    /// ban peer
    #[method(name = "ban_peer")]
    fn ban_peer(&self, peer_id: String, ban: bool) -> RpcResult<()>;
}

pub use NetworkManagerApiClient as NetworkManagerApiRpcClient;
pub use NetworkManagerApiServer as NetworkManagerApiRpcServer;

/// Build jsonrpsee methods from legacy `NetworkManagerApi`.
pub fn network_manager_methods<T>(api: T) -> std::result::Result<Methods, RegisterMethodError>
where
    T: NetworkManagerApiServer + Send + Sync + 'static,
{
    Ok(NetworkManagerApiServer::into_rpc(api).into())
}
