// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod block_connector;
pub mod block_sync;
pub mod download;
pub mod helper;
pub mod state_sync;
mod sync;
mod sync_metrics;
mod sync_task;
pub mod task;
pub mod txn_sync;

pub use download::Downloader;
pub use sync::SyncService;

mod sync_event_handle;
pub mod verified_rpc_client;

use dyn_clone::DynClone;
use starcoin_crypto::HashValue;

#[async_trait::async_trait]
pub trait StateSyncReset: DynClone + Send + Sync {
    async fn reset(
        &self,
        state_root: HashValue,
        block_accumulator_root: HashValue,
        pivot_id: HashValue,
    );
}
