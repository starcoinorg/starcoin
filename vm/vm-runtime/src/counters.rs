use once_cell::sync::Lazy;
use prometheus::{HistogramOpts, HistogramVec, IntCounterVec, Opts};

pub static TXN_STATUS_COUNTERS: Lazy<IntCounterVec> = Lazy::new(|| {
    let opts = Opts::new("vm_txn_stats", "Counters of executed txn").namespace("starcoin");
    register_int_counter_vec!(opts, &["status"]).unwrap()
});

pub static TXN_EXECUTION_HISTOGRAM: Lazy<HistogramVec> = Lazy::new(|| {
    let opts =
        HistogramOpts::new("vm_execution", "Histogram of txn execution").namespace("starcoin");
    register_histogram_vec!(opts, &["api"]).unwrap()
});
