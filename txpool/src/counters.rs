use once_cell::sync::Lazy;
use prometheus::{HistogramVec, IntCounterVec, IntGauge, IntGaugeVec};

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

pub static TXPOOL_STATUS_GAUGE_VEC: Lazy<IntGaugeVec> = Lazy::new(|| {
    register_int_gauge_vec!("txpool_status", "Gauge of pool status", &["name"]).unwrap()
});

pub static TXPOOL_SERVICE_HISTOGRAM: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!("txpool_service", "Histogram of txpool service", &["api"]).unwrap()
});
