use starcoin_metrics::{
    register_int_counter, register_int_gauge, IntCounter, IntGauge, Opts, PrometheusError,
};

const SC_NS: &str = "starcoin";
const PRIFIX: &str = "starcoin_chain_";

#[derive(Clone)]
pub struct ChainMetrics {
    pub try_connect_count: IntCounter,
    pub duplicate_conn_count: IntCounter,
    pub rollback_count: IntCounter,
    pub broadcast_head_count: IntCounter,
    pub verify_fail_count: IntCounter,
    pub exe_block_total_time: IntCounter,
    pub branch_total_count: IntGauge,
}

impl ChainMetrics {
    pub fn register() -> Result<Self, PrometheusError> {
        let try_connect_count = register_int_counter!(Opts::new(
            format!("{}{}", PRIFIX, "try_connect_count"),
            "block try connect count".to_string()
        )
        .namespace(SC_NS))?;

        let duplicate_conn_count = register_int_counter!(Opts::new(
            format!("{}{}", PRIFIX, "duplicate_conn_count"),
            "block duplicate connect count".to_string()
        )
        .namespace(SC_NS))?;

        let rollback_count = register_int_counter!(Opts::new(
            format!("{}{}", PRIFIX, "rollback_count"),
            "chain rollback count".to_string()
        )
        .namespace(SC_NS))?;

        let broadcast_head_count = register_int_counter!(Opts::new(
            format!("{}{}", PRIFIX, "broadcast_head_count"),
            "chain broadcast head count".to_string()
        )
        .namespace(SC_NS))?;

        let verify_fail_count = register_int_counter!(Opts::new(
            format!("{}{}", PRIFIX, "verify_fail_count"),
            "block verify fail count".to_string()
        )
        .namespace(SC_NS))?;

        let exe_block_total_time = register_int_counter!(Opts::new(
            format!("{}{}", PRIFIX, "exe_block_total_time"),
            "execute block total time".to_string()
        )
        .namespace(SC_NS))?;

        let branch_total_count = register_int_gauge!(Opts::new(
            format!("{}{}", PRIFIX, "branch_total_count"),
            "branch total count".to_string()
        )
        .namespace(SC_NS))?;

        Ok(Self {
            try_connect_count,
            duplicate_conn_count,
            rollback_count,
            broadcast_head_count,
            verify_fail_count,
            exe_block_total_time,
            branch_total_count,
        })
    }
}
