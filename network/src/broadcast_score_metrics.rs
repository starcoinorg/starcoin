use once_cell::sync::Lazy;
use prometheus::default_registry;
use starcoin_metrics::{
    register_int_counter_vec, IntCounterVec, Opts, PrometheusError, UIntCounterVec,
};
use starcoin_types::peer_info::PeerId;

const SC_NS: &str = "starcoin";

pub static BROADCAST_SCORE_METRICS: Lazy<BroadcastScoreMetrics> =
    Lazy::new(|| BroadcastScoreMetrics::register().unwrap());

#[derive(Clone)]
pub struct BroadcastScoreMetrics {
    pub peer_broadcast_score: IntCounterVec,
    pub peer_broadcast_total_new_count: UIntCounterVec,
    pub peer_broadcast_total_old_count: UIntCounterVec,
}

impl BroadcastScoreMetrics {
    pub fn register() -> Result<Self, PrometheusError> {
        let peer_broadcast_score = register_int_counter_vec!(
            Opts::new("peer_broadcast_score", "peer broadcast score".to_string()).namespace(SC_NS),
            &["broadcast_score"]
        )?;

        let peer_broadcast_total_new_count = UIntCounterVec::new(
            Opts::new(
                "peer_broadcast_total_new_count",
                "total new count".to_string(),
            )
            .namespace(SC_NS),
            &["total_new_count"],
        )?;

        let peer_broadcast_total_old_count = UIntCounterVec::new(
            Opts::new(
                "peer_broadcast_total_old_count",
                "total old count".to_string(),
            )
            .namespace(SC_NS),
            &["total_old_count"],
        )?;

        default_registry().register(Box::new(peer_broadcast_total_new_count.clone()))?;
        default_registry().register(Box::new(peer_broadcast_total_old_count.clone()))?;

        Ok(Self {
            peer_broadcast_score,
            peer_broadcast_total_new_count,
            peer_broadcast_total_old_count,
        })
    }

    pub fn report_new(&self, peer: PeerId, score: i64) {
        self.peer_broadcast_score
            .with_label_values(&[&format!("peer-{:?}", peer)])
            .inc_by(score as u64);
        self.peer_broadcast_total_new_count
            .with_label_values(&[&format!("peer-{:?}", peer)])
            .inc();
    }

    pub fn report_expire(&self, peer: PeerId, score: i64) {
        self.peer_broadcast_score
            .with_label_values(&[&format!("peer-{:?}", peer)])
            .inc_by(score as u64);
        self.peer_broadcast_total_old_count
            .with_label_values(&[&format!("peer-{:?}", peer)])
            .inc();
    }
}
