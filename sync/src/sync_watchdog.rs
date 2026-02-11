// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::time::Instant;

#[derive(Clone, Debug)]
pub struct SyncWatchdogSnapshot {
    pub task_name: String,
    pub processed: u64,
    pub ok: u64,
    pub last_change: Instant,
}

pub fn update_watchdog_state(
    state: &mut Option<SyncWatchdogSnapshot>,
    task_name: String,
    processed: u64,
    ok: u64,
    now: Instant,
    stall_secs: u64,
) -> bool {
    match state.as_mut() {
        Some(snapshot)
            if snapshot.task_name != task_name
                || processed < snapshot.processed
                || ok < snapshot.ok =>
        {
            *snapshot = SyncWatchdogSnapshot {
                task_name,
                processed,
                ok,
                last_change: now,
            };
            false
        }
        Some(snapshot) => {
            if processed > snapshot.processed || ok > snapshot.ok {
                snapshot.processed = processed;
                snapshot.ok = ok;
                snapshot.last_change = now;
                return false;
            }
            if now.duration_since(snapshot.last_change).as_secs() >= stall_secs {
                snapshot.last_change = now;
                snapshot.processed = processed;
                snapshot.ok = ok;
                return true;
            }
            false
        }
        None => {
            *state = Some(SyncWatchdogSnapshot {
                task_name,
                processed,
                ok,
                last_change: now,
            });
            false
        }
    }
}
