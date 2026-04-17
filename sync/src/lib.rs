// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![deny(clippy::arithmetic_side_effects)]
use std::sync::atomic::{AtomicBool, Ordering};

pub mod announcement;
pub mod block_connector;
pub mod store;
pub mod sync;
pub mod sync_metrics;
pub mod sync_watchdog;
pub mod tasks;
pub mod txn_sync;

pub mod parallel;
pub mod verified_rpc_client;

static SYNC_PROFILING_INFO_ENABLED: AtomicBool = AtomicBool::new(false);

pub(crate) fn set_sync_profiling_info_enabled(enabled: bool) {
    SYNC_PROFILING_INFO_ENABLED.store(enabled, Ordering::Relaxed);
}

pub(crate) fn sync_profiling_info_enabled() -> bool {
    SYNC_PROFILING_INFO_ENABLED.load(Ordering::Relaxed)
}
