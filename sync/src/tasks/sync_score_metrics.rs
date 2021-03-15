use network_api::PeerStrategy;
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
    sub_sync_target_count: IntGaugeVec,
    sub_sync_target_time: IntGaugeVec,
    sub_sync_target_avg_time: IntGaugeVec,
    sub_sync_target_peers: IntGaugeVec,
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

        let sub_sync_target_count = IntGaugeVec::new(
            Opts::new("sub_sync_target_count", "sub sync target count").namespace(SC_NS),
            &["sub_count"],
        )?;

        let sub_sync_target_time = IntGaugeVec::new(
            Opts::new("sub_sync_target_time", "sub sync target time").namespace(SC_NS),
            &["sub_time"],
        )?;

        let sub_sync_target_avg_time = IntGaugeVec::new(
            Opts::new("sub_sync_target_avg_time", "sub sync target avg time").namespace(SC_NS),
            &["sub_avg_time"],
        )?;

        let sub_sync_target_peers = IntGaugeVec::new(
            Opts::new("sub_sync_target_peers", "sub sync target peers").namespace(SC_NS),
            &["sub_peers"],
        )?;

        default_registry().register(Box::new(peer_sync_total_time.clone()))?;
        default_registry().register(Box::new(peer_sync_total_count.clone()))?;
        default_registry().register(Box::new(peer_sync_per_time.clone()))?;
        default_registry().register(Box::new(peer_sync_per_score.clone()))?;
        default_registry().register(Box::new(sub_sync_target_count.clone()))?;
        default_registry().register(Box::new(sub_sync_target_time.clone()))?;
        default_registry().register(Box::new(sub_sync_target_avg_time.clone()))?;
        default_registry().register(Box::new(sub_sync_target_peers.clone()))?;

        Ok(Self {
            peer_sync_total_score,
            peer_sync_total_time,
            peer_sync_total_count,
            peer_sync_per_time,
            peer_sync_per_score,
            sub_sync_target_count,
            sub_sync_target_time,
            sub_sync_target_avg_time,
            sub_sync_target_peers,
        })
    }

    pub fn update_metrics(&self, peer: PeerId, time: u32, score: i64) {
        self.peer_sync_total_score
            .with_label_values(&[&format!("peer-{:?}", peer)])
            .inc_by(score as u64);
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

    pub fn report_sub_sync_target_metrics(
        &self,
        peers: usize,
        strategy: PeerStrategy,
        count: i64,
        time: i64,
    ) {
        self.sub_sync_target_count
            .with_label_values(&[&format!("peer-{:?}", strategy)])
            .set(count);
        self.sub_sync_target_time
            .with_label_values(&[&format!("peer-{:?}", strategy)])
            .set(time);
        let (avg_time, _) = time.overflowing_div(count);
        self.sub_sync_target_avg_time
            .with_label_values(&[&format!("peer-{:?}", strategy)])
            .set(avg_time);
        self.sub_sync_target_peers
            .with_label_values(&[&format!("peer-{:?}", strategy)])
            .set(peers as i64);
    }
}
