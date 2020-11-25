// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

pub use self::gen_client::Client as SyncManagerClient;
use crate::FutureResult;
use jsonrpc_derive::rpc;
use starcoin_sync_api::SyncProgressReport;
use starcoin_types::peer_info::PeerId;
use starcoin_types::sync_status::SyncStatus;

#[rpc]
pub trait SyncManagerApi {
    #[rpc(name = "sync.status")]
    fn status(&self) -> FutureResult<SyncStatus>;

    #[rpc(name = "sync.cancel")]
    fn cancel(&self) -> FutureResult<()>;

    #[rpc(name = "sync.start")]
    /// if `force` is true, will cancel current task and start a new task.
    /// if peers is not empty, will try sync with the special peers.
    fn start(&self, force: bool, peers: Vec<PeerId>) -> FutureResult<()>;

    #[rpc(name = "sync.progress")]
    fn progress(&self) -> FutureResult<Option<SyncProgressReport>>;
}
