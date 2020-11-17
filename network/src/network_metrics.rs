use prometheus::{Error as PrometheusError, Histogram, IntGauge, Opts};

#[derive(Clone)]
pub struct NetworkMetrics {
    pub request_count: IntGauge,
    pub request_timeout_count: IntGauge,
    pub request_block_latency: Histogram,
}

impl NetworkMetrics {
    pub fn register() -> Result<Self, PrometheusError> {
        let request_count = register_int_gauge!(
            Opts::new("request_count", "rpc request count").namespace("starcoin")
        )?;

        let request_timeout_count =
            register_int_gauge!(
                Opts::new("request_timeout_count", "request timeout count").namespace("starcoin")
            )?;
        let request_block_latency =
            register_histogram!("request_block_latency", "request_block_latency")?;
        Ok(Self {
            request_count,
            request_timeout_count,
            request_block_latency,
        })
    }
}
