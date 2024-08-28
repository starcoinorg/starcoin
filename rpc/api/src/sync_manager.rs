// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

pub use self::gen_client::Client as SyncManagerClient;
use crate::{types::SyncStatusView, FutureResult};
use network_api::PeerStrategy;
use network_p2p_types::peer_id::PeerId;
use openrpc_derive::openrpc;
use starcoin_sync_api::{PeerScoreResponse, SyncProgressReport};

#[openrpc]
pub trait SyncManagerApi {
    #[rpc(name = "sync.status")]
    fn status(&self) -> FutureResult<SyncStatusView>;

    #[rpc(name = "sync.cancel")]
    fn cancel(&self) -> FutureResult<()>;

    #[rpc(name = "sync.start")]
    /// if `force` is true, will cancel current task and start a new task.
    /// if peers is not empty, will try sync with the special peers.
    fn start(
        &self,
        force: bool,
        peers: Vec<PeerId>,
        skip_pow_verify: bool,
        strategy: Option<PeerStrategy>,
    ) -> FutureResult<()>;

    #[rpc(name = "sync.progress")]
    fn progress(&self) -> FutureResult<Option<SyncProgressReport>>;

    #[rpc(name = "sync.score")]
    fn peer_score(&self) -> FutureResult<PeerScoreResponse>;
}
#[test]
fn test() {
    let schema = self::gen_schema();
    let j = serde_json::to_string_pretty(&schema).unwrap();
    println!("{}", j);
}
