use once_cell::sync::Lazy;
use prometheus::{IntCounterVec, IntGauge};

/// Counter of txn status in tx_pool
pub static TXN_STATUS_COUNTERS: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "txpool_txn_stats",
        "Counters of how many txn's stats in tx pool",
        &["status"]
    )
    .unwrap()
});

pub static TXPOOL_TXNS_GAUGE: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!("txpool_txn_nums", "Counter of how many txns in txpool").unwrap()
});
