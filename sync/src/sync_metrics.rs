use once_cell::sync::Lazy;
use starcoin_metrics::{
    register_histogram_vec, register_int_counter_vec, HistogramOpts, HistogramVec, IntCounterVec,
    Opts, PrometheusError,
};

const SC_NS: &str = "starcoin";
const PREFIX: &str = "sync_";

pub const LABEL_BLOCK: &str = "block";
pub const LABEL_BLOCK_BODY: &str = "body";
pub const LABEL_HASH: &str = "hash";
pub const LABEL_STATE: &str = "state";
pub const LABEL_TXN_INFO: &str = "txn_info";
pub const LABEL_ACCUMULATOR: &str = "accumulator";

pub static SYNC_METRICS: Lazy<SyncMetrics> = Lazy::new(|| SyncMetrics::register().unwrap());

#[derive(Clone)]
pub struct SyncMetrics {
    pub sync_total_count: IntCounterVec,
    pub sync_succ_count: IntCounterVec,
    pub sync_fail_count: IntCounterVec,
    pub sync_verify_fail_count: IntCounterVec,
    pub sync_done_time: HistogramVec,
    pub sync_get_block_time: HistogramVec,
    pub sync_get_block_ids_time: HistogramVec,
    pub sync_apply_block_time: HistogramVec,
    pub sync_count: IntCounterVec,
    pub sync_done_count: IntCounterVec,
}

impl SyncMetrics {
    pub fn register() -> Result<Self, PrometheusError> {
        let sync_total_count = register_int_counter_vec!(
            Opts::new(
                format!("{}{}", PREFIX, "sync_total_count"),
                "sync total count".to_string()
            )
            .namespace(SC_NS),
            &["sync_total_count"]
        )?;

        let sync_succ_count = register_int_counter_vec!(
            Opts::new(
                format!("{}{}", PREFIX, "sync_succ_count"),
                "sync succ count".to_string()
            )
            .namespace(SC_NS),
            &["sync_succ_count"]
        )?;

        let sync_fail_count = register_int_counter_vec!(
            Opts::new(
                format!("{}{}", PREFIX, "sync_fail_count"),
                "sync fail count".to_string()
            )
            .namespace(SC_NS),
            &["sync_fail_count"]
        )?;

        let sync_verify_fail_count = register_int_counter_vec!(
            Opts::new(
                format!("{}{}", PREFIX, "sync_verify_fail_count"),
                "sync verify fail count".to_string()
            )
            .namespace(SC_NS),
            &["sync_verify_fail_count"]
        )?;

        let sync_done_time = register_histogram_vec!(
            HistogramOpts::new(
                format!("{}{}", PREFIX, "sync_done_time"),
                "sync done time".to_string()
            )
            .namespace(SC_NS),
            &["sync_done_time"]
        )?;
        let sync_get_block_time = register_histogram_vec!(
            HistogramOpts::new(
                format!("{}{}", PREFIX, "sync_get_block_time"),
                "sync get_block time".to_string()
            )
            .namespace(SC_NS),
            &["sync_get_block_time"]
        )?;
        let sync_get_block_ids_time = register_histogram_vec!(
            HistogramOpts::new(
                format!("{}{}", PREFIX, "sync_get_block_ids_time"),
                "sync get_block_ids time".to_string()
            )
            .namespace(SC_NS),
            &["sync_get_block_ids_time"]
        )?;
        let sync_apply_block_time = register_histogram_vec!(
            HistogramOpts::new(
                format!("{}{}", PREFIX, "sync_apply_block_time"),
                "sync apply_block time".to_string()
            )
            .namespace(SC_NS),
            &["sync_apply_block_time"]
        )?;

        let sync_count = register_int_counter_vec!(
            Opts::new(
                format!("{}{}", PREFIX, "sync_count"),
                "sync count".to_string()
            )
            .namespace(SC_NS),
            &["sync_count"]
        )?;

        let sync_done_count = register_int_counter_vec!(
            Opts::new(
                format!("{}{}", PREFIX, "sync_done_count"),
                "sync done count".to_string()
            )
            .namespace(SC_NS),
            &["sync_done_count"]
        )?;

        Ok(Self {
            sync_total_count,
            sync_succ_count,
            sync_fail_count,
            sync_verify_fail_count,
            sync_done_time,
            sync_get_block_time,
            sync_get_block_ids_time,
            sync_apply_block_time,
            sync_count,
            sync_done_count,
        })
    }
}
