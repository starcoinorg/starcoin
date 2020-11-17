use once_cell::sync::Lazy;
use starcoin_metrics::{register_int_gauge, IntGauge, Opts, PrometheusError};

pub static BLOCK_RELAYER_METRICS: Lazy<BlockRelayerMetrics> =
    Lazy::new(|| BlockRelayerMetrics::register().unwrap());

#[derive(Clone)]
pub struct BlockRelayerMetrics {
    pub txns_filled_from_network: IntGauge,
    pub txns_filled_from_txpool: IntGauge,
    pub txns_filled_from_prefill: IntGauge,
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
        Ok(Self {
            txns_filled_from_network,
            txns_filled_from_txpool,
            txns_filled_from_prefill,
        })
    }
}
