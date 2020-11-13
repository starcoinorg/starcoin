// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

pub use self::gen_client::Client as SyncManagerClient;
use crate::FutureResult;
use jsonrpc_derive::rpc;
use starcoin_sync_api::TaskProgressReport;
use starcoin_types::sync_status::SyncStatus;

#[rpc]
pub trait SyncManagerApi {
    #[rpc(name = "sync.status")]
    fn status(&self) -> FutureResult<SyncStatus>;

    #[rpc(name = "sync.cancel")]
    fn cancel(&self) -> FutureResult<()>;

    #[rpc(name = "sync.start")]
    fn start(&self, force: bool) -> FutureResult<()>;

    #[rpc(name = "sync.progress")]
    fn progress(&self) -> FutureResult<Option<TaskProgressReport>>;
}
