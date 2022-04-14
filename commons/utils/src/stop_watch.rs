// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use once_cell::sync::Lazy;
use parking_lot::Mutex;
use simple_stopwatch::Stopwatch;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};

static G_WATCH_STATUS: AtomicBool = AtomicBool::new(false);

pub type WatchName = &'static str;

pub const CHAIN_WATCH_NAME: WatchName = "chain";

static G_DEFAULT_WATCH: Lazy<Mutex<Stopwatch>> = Lazy::new(|| Mutex::new(Stopwatch::start_new()));
static G_WATCH_MAP: Lazy<HashMap<WatchName, Mutex<Stopwatch>>> = Lazy::new(|| {
    let mut watch_map = HashMap::new();
    watch_map.insert(CHAIN_WATCH_NAME, Mutex::new(Stopwatch::start_new()));
    watch_map
});

/// Watch some method handle time.
pub fn watch(watch_name: &str, label: &str) {
    if G_WATCH_STATUS.load(Ordering::SeqCst) {
        let stop_watch = match G_WATCH_MAP.get(watch_name) {
            Some(stop_watch) => stop_watch,
            None => &G_DEFAULT_WATCH,
        };
        let mut watch = stop_watch.lock();
        watch.restart();
        println!("{:?}: {:?}", label, watch.ns());
    }
}

/// Start watching.
pub fn start_watch() {
    let _ = G_WATCH_STATUS
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::Relaxed)
        .unwrap_or_else(|x| x);
}

/// Stop watching.
pub fn stop_watch() {
    let _ = G_WATCH_STATUS
        .compare_exchange(true, false, Ordering::SeqCst, Ordering::Relaxed)
        .unwrap_or_else(|x| x);
}
