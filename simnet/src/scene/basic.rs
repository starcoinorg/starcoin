use super::{harness::drive_harness, GhostAdpter};
use crate::{BlkStream, DAGGenCfg, Miner};
use proptest::prelude::*;

fn deterministic_cfg() -> DAGGenCfg {
    DAGGenCfg {
        total_time: 10_000,
        block_interval: 200,
        miners: vec![
            Miner {
                name: "alice",
                hash_power_ratio: 0.5,
                network_delay: 1000,
                is_attacker: false,
                hide_blocks_plan: vec![],
            },
            Miner {
                name: "bob",
                hash_power_ratio: 0.3,
                network_delay: 1200,
                is_attacker: false,
                hide_blocks_plan: vec![],
            },
            Miner {
                name: "carol",
                hash_power_ratio: 0.2,
                network_delay: 180,
                is_attacker: false,
                hide_blocks_plan: vec![],
            },
        ],
    }
}

fn weighted_cfg() -> DAGGenCfg {
    DAGGenCfg {
        total_time: 500_000,
        block_interval: 500,
        miners: vec![
            Miner {
                name: "alice",
                hash_power_ratio: 0.5,
                network_delay: 200,
                is_attacker: false,
                hide_blocks_plan: vec![],
            },
            Miner {
                name: "bob",
                hash_power_ratio: 0.3,
                network_delay: 200,
                is_attacker: false,
                hide_blocks_plan: vec![],
            },
            Miner {
                name: "carol",
                hash_power_ratio: 0.2,
                network_delay: 200,
                is_attacker: false,
                hide_blocks_plan: vec![],
            },
        ],
    }
}


fn arb_weights(count: usize) -> impl Strategy<Value = Vec<f64>> {
    prop::collection::vec(0.1f64..1.0, count).prop_map(|mut raw| {
        let sum: f64 = raw.iter().sum();
        for weight in &mut raw {
            *weight /= sum;
        }
        raw
    })
}

const RANDOM_MINER_NAMES: [&str; 10] = [
    "mh0", "mh1", "mh2", "mh3", "mh4", "mh5", "mh6", "mh7", "mh8", "mh9",
];

fn cfg_from_weights(weights: &[f64]) -> DAGGenCfg {
    let miners = weights
        .iter()
        .enumerate()
        .map(|(idx, weight)| Miner {
            name: RANDOM_MINER_NAMES[idx % RANDOM_MINER_NAMES.len()],
            hash_power_ratio: *weight,
            network_delay: 200,
            is_attacker: false,
            hide_blocks_plan: vec![],
        })
        .collect();

    DAGGenCfg {
        total_time: 600_000,
        block_interval: 500,
        miners,
    }
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 6, .. ProptestConfig::default() })]

    #[test]
    fn simnet_honest_flow_maintains_topology(seed in any::<u64>()) {
        let cfg = deterministic_cfg();
        let mut gen = BlkStream::from_seed(cfg.clone(), seed);
        let events = gen.run();
        prop_assume!(!events.is_empty());

        let mut ghost = GhostAdpter::new(8, 3600, 8)
            .map_err(|e| TestCaseError::fail(e.to_string()))?;
        let accepted = drive_harness(&mut ghost, events.clone(), |_| false)
            .map_err(|e| TestCaseError::fail(e.to_string()))?;

        ghost
            .audit_consensus(&events)
            .map_err(|e| TestCaseError::fail(e.to_string()))?;
        let dot_path = ghost
            .dump_dot(&format!("honest_topology_{}", seed))
            .map_err(|e| TestCaseError::fail(e.to_string()))?;
        prop_assert!(dot_path.exists(), "dot file not generated");
        prop_assert_eq!(accepted, events.len());
    }

    #[test]
    fn block_counts_track_hash_power(seed in any::<u64>()) {
        let cfg = weighted_cfg();
        let mut gen = BlkStream::from_seed(cfg.clone(), seed);
        let events = gen.run();
        prop_assume!(events.len() > 40);

        let mut counts = vec![0usize; cfg.miners.len()];
        for ev in &events {
            counts[ev.miner_id] += 1;
        }

        let total_ratio: f64 = cfg.miners.iter().map(|m| m.hash_power_ratio).sum();
        let total_blocks = events.len() as f64;

        for (idx, miner) in cfg.miners.iter().enumerate() {
            let observed = counts[idx] as f64 / total_blocks;
            let expected = miner.hash_power_ratio / total_ratio;
            let diff = (observed - expected).abs();
            prop_assert!(
                diff < 0.15,
                "miner {} ratio drift too high: observed={:.3}, expected={:.3}, count={}, total={}",
                miner.name,
                observed,
                expected,
                counts[idx],
                events.len()
            );
        }
    }

    #[test]
    fn block_share_matches_random_hash_power(weights in arb_weights(10), seed in any::<u64>()) {
        let cfg = cfg_from_weights(&weights);
        let mut gen = BlkStream::from_seed(cfg.clone(), seed);
        let events = gen.run();
        prop_assume!(events.len() > 200);

        let mut counts = vec![0usize; cfg.miners.len()];
        for ev in &events {
            counts[ev.miner_id] += 1;
        }

        let total_blocks = events.len() as f64;
        for (idx, expected) in weights.iter().enumerate() {
            let observed = counts[idx] as f64 / total_blocks;
            let diff = (observed - expected).abs();
            let variance = (expected * (1.0 - expected) / total_blocks).max(1e-9);
            let tolerance = (variance.sqrt() * 3.0).max(0.05);
            prop_assert!(
                diff <= tolerance,
                "miner {idx} diff {diff:.3} exceeds tol {tol:.3}; observed={obs:.3}, expected={exp:.3}, total={}, counts={:?}",
                events.len(),
                counts,
                diff = diff,
                tol = tolerance,
                obs = observed,
                exp = expected
            );
        }
    }
}

