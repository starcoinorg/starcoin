use starcoin_sync::sync_watchdog::{update_watchdog_state, SyncWatchdogSnapshot};
use std::time::{Duration, Instant};

#[test]
fn test_watchdog_progress_updates_snapshot() {
    let now = Instant::now();
    let mut state = Some(SyncWatchdogSnapshot {
        task_name: "task".to_string(),
        processed: 1,
        ok: 1,
        last_change: now,
    });

    let should_restart = update_watchdog_state(&mut state, "task".to_string(), 2, 1, now, 10);
    assert!(!should_restart);
    let snapshot = state.expect("snapshot should exist");
    assert_eq!(snapshot.processed, 2);
    assert_eq!(snapshot.ok, 1);
}

#[test]
fn test_watchdog_detects_stall() {
    let now = Instant::now();
    let last_change = now
        .checked_sub(Duration::from_secs(11))
        .expect("Instant checked_sub failed");
    let mut state = Some(SyncWatchdogSnapshot {
        task_name: "task".to_string(),
        processed: 10,
        ok: 5,
        last_change,
    });

    let should_restart = update_watchdog_state(&mut state, "task".to_string(), 10, 5, now, 10);
    assert!(should_restart);
}
