use once_cell::sync::Lazy;
use starcoin_metrics::{
    default_registry, register_int_counter_vec, HistogramOpts, HistogramVec, IntCounterVec,
    IntGaugeVec, Opts, PrometheusError, UIntCounterVec,
};
use starcoin_types::peer_info::PeerId;

const SC_NS: &str = "starcoin";

pub static SYNC_SCORE_METRICS: Lazy<SyncScoreMetrics> =
    Lazy::new(|| SyncScoreMetrics::register().unwrap());

#[derive(Clone)]
pub struct SyncScoreMetrics {
    peer_sync_total_score: IntCounterVec,
    peer_sync_total_time: UIntCounterVec,
    peer_sync_total_count: UIntCounterVec,
    pub peer_sync_per_time: HistogramVec,
    peer_sync_per_score: IntGaugeVec,
}

impl SyncScoreMetrics {
    pub fn register() -> Result<Self, PrometheusError> {
        let peer_sync_total_score = register_int_counter_vec!(
            Opts::new("peer_sync_total_score", "total score".to_string()).namespace(SC_NS),
            &["total_score"]
        )?;

        let peer_sync_total_time = UIntCounterVec::new(
            Opts::new("peer_sync_total_time", "total time".to_string()).namespace(SC_NS),
            &["total_time"],
        )?;

        let peer_sync_total_count = UIntCounterVec::new(
            Opts::new("peer_sync_total_count", "total count".to_string()).namespace(SC_NS),
            &["total_count"],
        )?;

        let peer_sync_per_time = HistogramVec::new(
            HistogramOpts::new("peer_sync_per_time", "per time").namespace(SC_NS),
            &["per_time"],
        )?;

        let peer_sync_per_score = IntGaugeVec::new(
            Opts::new("peer_sync_per_score", "per score").namespace(SC_NS),
            &["per_score"],
        )?;

        default_registry().register(Box::new(peer_sync_total_time.clone()))?;
        default_registry().register(Box::new(peer_sync_total_count.clone()))?;
        default_registry().register(Box::new(peer_sync_per_time.clone()))?;
        default_registry().register(Box::new(peer_sync_per_score.clone()))?;

        Ok(Self {
            peer_sync_total_score,
            peer_sync_total_time,
            peer_sync_total_count,
            peer_sync_per_time,
            peer_sync_per_score,
        })
    }

    pub fn update_metrics(&self, peer: PeerId, time: u32, score: i64) {
        self.peer_sync_total_score
            .with_label_values(&[&format!("peer-{:?}", peer)])
            .inc_by(score);
        self.peer_sync_total_time
            .with_label_values(&[&format!("peer-{:?}", peer)])
            .inc_by(time as u64);
        self.peer_sync_total_count
            .with_label_values(&[&format!("peer-{:?}", peer)])
            .inc();
        self.peer_sync_per_score
            .with_label_values(&[&format!("peer-{:?}", peer)])
            .set(score);
    }
}
