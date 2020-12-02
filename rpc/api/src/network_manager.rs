// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

pub use self::gen_client::Client as NetworkManagerClient;
use crate::FutureResult;
use jsonrpc_derive::rpc;
use network_p2p_types::network_state::NetworkState;
use starcoin_types::peer_info::{Multiaddr, PeerId};

#[rpc]
pub trait NetworkManagerApi {
    #[rpc(name = "network_manager.state")]
    fn state(&self) -> FutureResult<NetworkState>;

    #[rpc(name = "network_manager.known_peers")]
    fn known_peers(&self) -> FutureResult<Vec<PeerId>>;

    #[rpc(name = "network_manager.get_address")]
    fn get_address(&self, peer_id: String) -> FutureResult<Vec<Multiaddr>>;

    #[rpc(name = "network_manager.add_peer")]
    fn add_peer(&self, peer: String) -> FutureResult<()>;
}
