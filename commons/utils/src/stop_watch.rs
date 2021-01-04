// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use once_cell::sync::Lazy;
use parking_lot::Mutex;
use simple_stopwatch::Stopwatch;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};

static WATCH_STATUS: AtomicBool = AtomicBool::new(false);

pub type WatchName = &'static str;

pub const DEFAULT_WATCH_NAME: WatchName = "default";
pub const CHAIN_WATCH_NAME: WatchName = "chain";

static WATCH_MAP: Lazy<HashMap<WatchName, Mutex<Stopwatch>>> = Lazy::new(|| {
    let mut watch_map = HashMap::new();
    watch_map.insert(DEFAULT_WATCH_NAME, Mutex::new(Stopwatch::start_new()));
    watch_map.insert(CHAIN_WATCH_NAME, Mutex::new(Stopwatch::start_new()));
    watch_map
});

/// Watch some method handle time.
pub fn watch(watch_name: &str, label: &str) {
    if WATCH_STATUS.load(Ordering::SeqCst) {
        let stop_watch = match WATCH_MAP.get(watch_name) {
            Some(stop_watch) => stop_watch,
            None => WATCH_MAP.get(DEFAULT_WATCH_NAME).unwrap(),
        };
        let mut watch = stop_watch.lock();
        watch.restart();
        println!("{:?}: {:?}", label, watch.ns());
    }
}

/// Start watching.
pub fn start_watch() {
    WATCH_STATUS.compare_and_swap(false, true, Ordering::SeqCst);
}

/// Stop watching.
pub fn stop_watch() {
    WATCH_STATUS.compare_and_swap(true, false, Ordering::SeqCst);
}
