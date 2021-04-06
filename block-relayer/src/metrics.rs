use once_cell::sync::Lazy;
use starcoin_metrics::{
    default_registry, register_histogram, register_int_gauge, register_uint_gauge, Histogram,
    IntGauge, Opts, PrometheusError, UIntCounterVec, UIntGauge,
};

pub static BLOCK_RELAYER_METRICS: Lazy<BlockRelayerMetrics> =
    Lazy::new(|| BlockRelayerMetrics::register().expect("BlockRelayerMetrics register should ok."));

#[derive(Clone)]
pub struct BlockRelayerMetrics {
    pub txns_filled_from_network: IntGauge,
    pub txns_filled_from_txpool: IntGauge,
    pub txns_filled_from_prefill: IntGauge,
    pub txns_filled_time: Histogram,
    pub block_broadcast_time: Histogram,
    pub txns_filled_failed: UIntCounterVec,
    pub block_txns_count: UIntGauge,
}

impl BlockRelayerMetrics {
    pub fn register() -> Result<Self, PrometheusError> {
        let txns_filled_from_network = register_int_gauge!(Opts::new(
            "txns_filled_from_network",
            "Count of block filled transactions from network"
        )
        .namespace("starcoin"))?;
        let txns_filled_from_txpool = register_int_gauge!(Opts::new(
            "txns_filled_from_txpool",
            "Count of block filled transactions from txpool"
        )
        .namespace("starcoin"))?;
        let txns_filled_from_prefill = register_int_gauge!(Opts::new(
            "txns_filled_from_prefill",
            "Count of block filled transactions from prefill"
        )
        .namespace("starcoin"))?;
        let txns_filled_time =
            register_histogram!("starcoin_txns_filled_time", "txns filled time")?;
        let block_broadcast_time = register_histogram!("block_broadcast", "block broadcast time.")?;

        let txns_filled_failed = UIntCounterVec::new(
            Opts::new(
                "starcoin_txns_filled_failed",
                "txns filled failed".to_string(),
            )
            .namespace("starcoin"),
            &["type"],
        )?;
        let broadcast_txns_count =
            register_uint_gauge!("starcoin_broadcast_txns_count", "broadcast txns count.")?;
        default_registry().register(Box::new(txns_filled_failed.clone()))?;
        Ok(Self {
            txns_filled_from_network,
            txns_filled_from_txpool,
            txns_filled_from_prefill,
            txns_filled_time,
            block_broadcast_time,
            txns_filled_failed,
            block_txns_count: broadcast_txns_count,
        })
    }
}
