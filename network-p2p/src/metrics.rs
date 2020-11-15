use starcoin_metrics::{IntCounterVec, IntGauge, Opts, PrometheusError, UIntGaugeVec};

#[derive(Clone)]
pub struct Metrics {
    pub network_per_sec_bytes: UIntGaugeVec,
    pub connections_closed_total: IntCounterVec,
    pub connections_opened_total: IntCounterVec,
    pub peers_count: IntGauge,
}

impl Metrics {
    pub fn register() -> Result<Self, PrometheusError> {
        let network_per_sec_bytes = register_uint_gauge_vec!(
            Opts::new(
                "sub_libp2p_network_per_sec_bytes",
                "Average bandwidth usage per second"
            )
            .namespace("starcoin"),
            &["direction"]
        )?;

        let connections_opened_total = register_int_counter_vec!(
            Opts::new(
                "network_connection_opened",
                "Counters of how many connections opened",
            )
            .namespace("starcoin"),
            &["network_connection_opened"]
        )?;

        let connections_closed_total = register_int_counter_vec!(
            Opts::new(
                "network_connection_closed",
                "Counters of how many connections closed",
            )
            .namespace("starcoin"),
            &["direction", "reason"]
        )?;

        let peers_count =
            register_int_gauge!(Opts::new("peers_count", "peers count").namespace("starcoin"))?;

        Ok(Self {
            network_per_sec_bytes,
            connections_closed_total,
            connections_opened_total,
            peers_count,
        })
    }
}
