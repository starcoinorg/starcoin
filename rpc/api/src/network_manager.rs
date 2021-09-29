// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

pub use self::gen_client::Client as NetworkManagerClient;
use crate::types::StrView;
use crate::FutureResult;
use jsonrpc_core::Result;
use jsonrpc_derive::rpc;
use network_p2p_types::network_state::NetworkState;
use starcoin_types::peer_info::{Multiaddr, PeerId};
use std::borrow::Cow;

#[rpc(client, server, schema)]
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

#[test]
fn test() {
    let schema = rpc_impl_NetworkManagerApi::gen_client::Client::gen_schema();
    let j = serde_json::to_string_pretty(&schema).unwrap();
    println!("{}", j);
}
