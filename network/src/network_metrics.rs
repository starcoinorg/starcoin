use prometheus::{Error as PrometheusError, IntGauge, Opts};

#[derive(Clone)]
pub struct NetworkMetrics {
    pub request_count: IntGauge,
    pub request_timeout_count: IntGauge,
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

        Ok(Self {
            request_count,
            request_timeout_count,
        })
    }
}
