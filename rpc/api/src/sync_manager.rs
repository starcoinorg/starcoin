// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

pub use self::gen_client::Client as SyncManagerClient;
use crate::FutureResult;
use jsonrpc_derive::rpc;
use network_api::PeerStrategy;
use starcoin_sync_api::{PeerScoreResponse, SyncProgressReport};
use starcoin_types::peer_info::PeerId;
use starcoin_types::sync_status::SyncStatus;

#[rpc(client, server, schema)]
pub trait SyncManagerApi {
    #[rpc(name = "sync.status")]
    fn status(&self) -> FutureResult<SyncStatus>;

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
    let schema = rpc_impl_SyncManagerApi::gen_client::Client::gen_schema();
    let j = serde_json::to_string_pretty(&schema).unwrap();
    println!("{}", j);
}
