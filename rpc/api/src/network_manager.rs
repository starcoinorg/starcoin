// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

pub use self::gen_client::Client as NetworkManagerClient;
use crate::types::StrView;
use crate::FutureResult;
use jsonrpc_core::Result;
use jsonrpsee::{core::RegisterMethodError, Methods, RpcModule};
use network_p2p_types::network_state::NetworkState;
use network_p2p_types::peer_id::PeerId;
use network_types::peer_info::Multiaddr;
use openrpc_derive::openrpc;
use std::borrow::Cow;
use std::sync::Arc;

#[openrpc]
pub trait NetworkManagerApi {
    #[rpc(name = "network_manager.state")]
    fn state(&self) -> FutureResult<NetworkState>;

    #[rpc(name = "network_manager.known_peers")]
    fn known_peers(&self) -> FutureResult<Vec<PeerId>>;

    #[rpc(name = "network_manager.get_address")]
    fn get_address(&self, peer_id: String) -> FutureResult<Vec<Multiaddr>>;

    #[rpc(name = "network_manager.add_peer")]
    fn add_peer(&self, peer: String) -> FutureResult<()>;

    /// Call peer's network rpc method.
    #[rpc(name = "network_manager.call")]
    fn call_peer(
        &self,
        peer_id: String,
        rpc_method: Cow<'static, str>,
        message: StrView<Vec<u8>>,
    ) -> FutureResult<StrView<Vec<u8>>>;

    /// Set peer reputation
    #[rpc(name = "network_manager.set_peer_reput")]
    fn set_peer_reputation(&self, peer_id: String, reputation: i32) -> FutureResult<()>;

    /// ban peer
    #[rpc(name = "network_manager.ban_peer")]
    fn ban_peer(&self, peer_id: String, ban: bool) -> Result<()>;
}

/// Build jsonrpsee methods from legacy `NetworkManagerApi`.
pub fn network_manager_methods<T>(api: T) -> std::result::Result<Methods, RegisterMethodError>
where
    T: NetworkManagerApi + Send + Sync + 'static,
{
    let mut module = RpcModule::new(Arc::new(api));

    module.register_async_method("network_manager.state", |_, api, _| async move {
        api.state().await.map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("network_manager.known_peers", |_, api, _| async move {
        api.known_peers().await.map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("network_manager.get_address", |params, api, _| async move {
        let peer_id: String = params.one()?;
        api.get_address(peer_id).await.map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("network_manager.add_peer", |params, api, _| async move {
        let peer: String = params.one()?;
        api.add_peer(peer).await.map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("network_manager.call", |params, api, _| async move {
        let (peer_id, rpc_method, message): (String, String, StrView<Vec<u8>>) = params.parse()?;
        api.call_peer(peer_id, Cow::Owned(rpc_method), message)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("network_manager.set_peer_reput", |params, api, _| async move {
        let (peer_id, reputation): (String, i32) = params.parse()?;
        api.set_peer_reputation(peer_id, reputation)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;

    module.register_method("network_manager.ban_peer", |params, api, _| {
        let (peer_id, ban): (String, bool) = params.parse()?;
        api.ban_peer(peer_id, ban).map_err(crate::map_jsonrpc_err)
    })?;

    Ok(module.into())
}

#[test]
fn test() {
    let schema = self::gen_schema();
    let j = serde_json::to_string_pretty(&schema).unwrap();
    println!("{}", j);
}
