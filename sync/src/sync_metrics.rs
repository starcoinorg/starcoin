use once_cell::sync::Lazy;
use starcoin_metrics::{
    default_registry, register_histogram_vec, HistogramOpts, HistogramVec, Opts, PrometheusError,
    UIntCounterVec,
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
    pub sync_get_block_ids_time: HistogramVec,
    pub sync_apply_block_time: HistogramVec,
    pub sync_times: UIntCounterVec,
}

impl SyncMetrics {
    pub fn register() -> Result<Self, PrometheusError> {
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

        let sync_times = UIntCounterVec::new(
            Opts::new(
                format!("{}{}", PREFIX, "sync_times"),
                "sync times".to_string(),
            )
            .namespace(SC_NS),
            &["type"],
        )?;

        default_registry().register(Box::new(sync_times.clone()))?;

        Ok(Self {
            sync_get_block_ids_time,
            sync_apply_block_time,
            sync_times,
        })
    }
}
