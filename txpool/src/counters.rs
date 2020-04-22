use once_cell::sync::Lazy;
use prometheus::IntCounterVec;

/// Counter of txn status in tx_pool
pub static TXN_STATUS_COUNTERS: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "txpool_txn_stats",
        "Counters of how many txn's stats in tx pool",
        &["added", "rejected", "dropped", "invaid", "canceled", "culled"]
    )
    .unwrap()
});
