use once_cell::sync::Lazy;
use prometheus::{HistogramOpts, HistogramVec, IntGauge, IntGaugeVec, Opts};

pub static TXPOOL_TXNS_GAUGE: Lazy<IntGauge> = Lazy::new(|| {
    let opts =
        Opts::new("txpool_txn_nums", "Counter of how many txns in txpool").namespace("starcoin");
    register_int_gauge!(opts).unwrap()
});

pub static TXPOOL_STATUS_GAUGE_VEC: Lazy<IntGaugeVec> = Lazy::new(|| {
    let opts = Opts::new("txpool_status", "Gauge of pool status").namespace("starcoin");
    register_int_gauge_vec!(opts, &["name"]).unwrap()
});

pub static TXPOOL_SERVICE_HISTOGRAM: Lazy<HistogramVec> = Lazy::new(|| {
    let opts =
        HistogramOpts::new("txpool_service", "Histogram of txpool service").namespace("starcoin");
    register_histogram_vec!(opts, &["api"]).unwrap()
});
