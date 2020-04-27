// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use once_cell::sync::Lazy;
use starcoin_metrics::{
    register_histogram_vec, register_int_counter_vec, HistogramVec, IntCounterVec,
};

pub static RPC_COUNTERS: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "starcoin_rpc",
        "Counters of how many rpc request",
        &["type", "method", "code"]
    )
    .unwrap()
});

pub static RPC_HISTOGRAMS: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!("starcoin_rpc_time", "Histogram of rpc request", &["method"]).unwrap()
});
