use super::{harness::drive_harness, GhostAdpter};
use crate::{BlkStream, DAGGenCfg, HideBlocksBurst, Miner};
use starcoin_crypto::HashValue;
use std::collections::{HashMap, HashSet};

/// Computes the probability that an attacker with hash power share `q` can
/// successfully execute a common-prefix violation after `depth` confirmations.
///
/// Implements the tail bound from the Bitcoin whitepaper, where
/// `λ = depth * q / p` and `p = 1 - q`.
fn attacker_success_probability(q: f64, depth: u32) -> f64 {
    assert!(
        (0.0..0.5).contains(&q),
        "attacker share must be in (0, 0.5)"
    );
    let p = 1.0 - q;
    let lambda = depth as f64 * q / p;

    // Poisson PMF initial term for k = 0.
    let mut poisson = (-lambda).exp();
    let mut cumulative = 1.0;

    for k in 0..=depth {
        let attacker_gap = depth - k;
        let ratio = (q / p).powi(attacker_gap as i32);
        cumulative -= poisson * (1.0 - ratio);

        if k < depth {
            let next_k = (k + 1) as f64;
            poisson *= lambda / next_k;
        }
    }

    cumulative.clamp(0.0, 1.0)
}

/// Finds the minimal confirmation depth such that the Bitcoin whitepaper
/// bound drops below `epsilon`.
fn minimal_confirmation_depth(q: f64, epsilon: f64) -> u32 {
    assert!(epsilon > 0.0 && epsilon < 1.0, "epsilon must be in (0, 1)");
    let mut depth = 1u32;
    loop {
        let prob = attacker_success_probability(q, depth);
        if prob <= epsilon {
            return depth;
        }
        depth += 1;
        assert!(depth < 1_000, "depth search exceeded practical bounds");
    }
}

#[test]
fn bitcoin_reference_depths_align() {
    let cases = vec![
        (0.1, 1e-2, 4u32),
        (0.1, 1e-3, 5u32),
        (0.1, 1e-6, 11u32),
        (0.2, 1e-4, 14u32),
        (0.2, 1e-6, 21u32),
        (0.3, 1e-4, 32u32),
    ];

    for (q, eps, expected_depth) in cases {
        let depth = minimal_confirmation_depth(q, eps);
        assert_eq!(depth, expected_depth, "q={q}, eps={eps}");
    }
}

#[test]
fn simnet_observed_failures_do_not_exceed_bitcoin_bound() {
    let cfg = dag_cfg_for_attacker(0.2);
    let stats = simulate_stats(&cfg, &[3, 7, 11, 19]);
    let (min_gap, rate) =
        report_gap_stats("q=0.20", &stats, 1e-4, 6).expect("no stable confirmation");
    println!(
        "common-prefix baseline (q=0.20): min_gap={} failure_rate={:.5e}",
        min_gap, rate
    );
}

#[test]
fn forced_common_prefix_violation() {
    let cfg = DAGGenCfg {
        total_time: 80_000,
        block_interval: 100,
        miners: vec![
            Miner {
                name: "honest_a",
                hash_power_ratio: 0.6,
                network_delay: 80,
                is_attacker: false,
                hide_blocks_plan: vec![],
            },
            Miner {
                name: "honest_b",
                hash_power_ratio: 0.2,
                network_delay: 120,
                is_attacker: false,
                hide_blocks_plan: vec![],
            },
            Miner {
                name: "adv",
                hash_power_ratio: 0.2,
                network_delay: 30,
                is_attacker: true,
                hide_blocks_plan: vec![HideBlocksBurst {
                    start_time_sec: 2,
                    block_count: 40,
                    release_time_sec: 40,
                    release_interval_ms: 5,
                }],
            },
        ],
    };

    let stats = simulate_stats(&cfg, &[7]);
    let forced_rate = stats.failure_rate(1).unwrap_or(0.0);
    println!("forced scenario gap=1 failure_rate={:.5e}", forced_rate);
    assert!(forced_rate > 0.0);
}

#[test]
fn hash_power_matrix() {
    let seeds = [2u64, 5, 8, 11];
    for attacker in [0.05, 0.1, 0.2, 0.3, 0.4] {
        let cfg = dag_cfg_for_attacker(attacker);
        let stats = simulate_stats(&cfg, &seeds);
        let (gap, rate) = report_gap_stats(&format!("q={:.2}", attacker), &stats, 1e-4, 10)
            .expect("no confirmations recorded");
        println!(
            "q={:.2} -> min_gap={} failure_rate={:.5e}",
            attacker, gap, rate
        );
    }
}

fn collect_chain(ghost: &GhostAdpter, mut tip: HashValue, limit: usize) -> Vec<HashValue> {
    let mut chain = Vec::new();
    for _ in 0..limit {
        chain.push(tip);
        if tip == ghost.genesis_id() {
            break;
        }
        let header = match ghost.header(tip) {
            Ok(header) => header,
            Err(_) => break,
        };
        tip = header.parent_hash();
    }
    chain
}

fn build_blue_score_map(ghost: &GhostAdpter) -> HashMap<HashValue, u64> {
    let mut scores = HashMap::new();

    if let Ok(Some(genesis_data)) = ghost.ghostdata(ghost.genesis_id()) {
        scores.insert(ghost.genesis_id(), genesis_data.blue_score);
    }

    for record in ghost.records() {
        if let Ok(Some(data)) = ghost.ghostdata(record.block_id) {
            scores.insert(record.block_id, data.blue_score);
        }
    }

    scores
}

fn collect_gap_stats(ghost: &GhostAdpter, blue_scores: &HashMap<HashValue, u64>) -> GapStats {
    let mut tracked: HashMap<HashValue, u64> = HashMap::new();
    let mut diff = GapDiff::default();

    for virtual_tip in ghost.virtual_tips() {
        let chain = collect_chain(
            ghost,
            virtual_tip.tip,
            ghost.records().len().saturating_add(1024),
        );
        if chain.len() < 2 {
            continue;
        }

        let Some(&tip_score) = blue_scores.get(&virtual_tip.tip) else {
            continue;
        };

        let chain_set: HashSet<_> = chain.iter().cloned().collect();

        let mut drops = Vec::new();
        for (hash, gap) in tracked.iter() {
            if !chain_set.contains(hash) && *gap > 0 {
                drops.push((*hash, *gap));
            }
        }

        for (hash, gap) in drops {
            tracked.remove(&hash);
            diff.record_failure(gap as usize);
        }

        for hash in chain.into_iter().skip(1) {
            let Some(&block_score) = blue_scores.get(&hash) else {
                continue;
            };
            let gap = tip_score.saturating_sub(block_score);
            if gap == 0 {
                continue;
            }

            let entry = tracked.entry(hash).or_insert(0);
            if gap > *entry {
                let from = (*entry as usize).saturating_add(1);
                diff.record_opportunity(from, gap as usize);
                *entry = gap;
            }
        }
    }

    diff.finish()
}

#[derive(Default)]
struct GapDiff {
    opp_diff: Vec<i64>,
    fail_diff: Vec<i64>,
}

impl GapDiff {
    fn ensure(&mut self, idx: usize) {
        if self.opp_diff.len() <= idx {
            self.opp_diff.resize(idx + 1, 0);
            self.fail_diff.resize(idx + 1, 0);
        }
    }

    fn record_opportunity(&mut self, start: usize, end: usize) {
        if start == 0 || start > end {
            return;
        }
        self.ensure(end + 1);
        self.opp_diff[start] += 1;
        self.opp_diff[end + 1] -= 1;
    }

    fn record_failure(&mut self, up_to: usize) {
        if up_to == 0 {
            return;
        }
        self.ensure(up_to + 1);
        self.fail_diff[1] += 1;
        self.fail_diff[up_to + 1] -= 1;
    }

    fn finish(self) -> GapStats {
        let mut opportunities = Vec::new();
        let mut failures = Vec::new();

        let mut run = 0i64;
        for val in self.opp_diff.into_iter().skip(1) {
            run += val;
            opportunities.push(run.max(0) as usize);
        }

        run = 0;
        for val in self.fail_diff.into_iter().skip(1) {
            run += val;
            failures.push(run.max(0) as usize);
        }

        GapStats {
            opportunities,
            failures,
        }
    }
}

#[derive(Default)]
struct GapStats {
    opportunities: Vec<usize>,
    failures: Vec<usize>,
}

impl GapStats {
    fn merge(&mut self, other: GapStats) {
        if self.opportunities.len() < other.opportunities.len() {
            self.opportunities.resize(other.opportunities.len(), 0);
            self.failures.resize(other.failures.len(), 0);
        }
        for (i, opp) in other.opportunities.into_iter().enumerate() {
            self.opportunities[i] += opp;
        }
        for (i, fail) in other.failures.into_iter().enumerate() {
            self.failures[i] += fail;
        }
    }

    fn failure_rate(&self, gap: u64) -> Option<f64> {
        if gap == 0 {
            return None;
        }
        let idx = (gap as usize).checked_sub(1)?;
        if idx >= self.opportunities.len() {
            return None;
        }
        let opp = self.opportunities[idx];
        if opp == 0 {
            None
        } else {
            Some(self.failures[idx] as f64 / opp as f64)
        }
    }

    fn opportunity_count(&self, gap: u64) -> Option<usize> {
        if gap == 0 {
            return None;
        }
        let idx = (gap as usize).checked_sub(1)?;
        self.opportunities.get(idx).copied()
    }
}
fn simulate_stats(cfg: &DAGGenCfg, seeds: &[u64]) -> GapStats {
    let mut aggregate = GapStats::default();
    for &seed in seeds {
        let mut generator = BlkStream::from_seed(cfg.clone(), seed);
        let events = generator.run();
        if events.is_empty() {
            continue;
        }
        let mut ghost = GhostAdpter::new(16, 3600, 16).expect("ghost init");
        let _ = drive_harness(&mut ghost, events.clone(), |miner_id| {
            cfg.miners[miner_id].is_attacker
        })
        .expect("drive harness");

        let blue_scores = build_blue_score_map(&ghost);
        let stats = collect_gap_stats(&ghost, &blue_scores);
        aggregate.merge(stats);
    }
    aggregate
}

fn report_gap_stats(
    label: &str,
    stats: &GapStats,
    epsilon: f64,
    max_gap: usize,
) -> Option<(u64, f64)> {
    println!("=== {} ===", label);
    let mut min_gap = None;
    for gap in 1..=max_gap {
        if let Some(rate) = stats.failure_rate(gap as u64) {
            let opp = stats.opportunity_count(gap as u64).unwrap_or(0);
            println!(
                "gap={} failure_rate={:.5e} opportunities={}",
                gap, rate, opp
            );
            if rate <= epsilon && min_gap.is_none() {
                min_gap = Some((gap as u64, rate));
            }
        }
    }
    min_gap
}

fn dag_cfg_for_attacker(attacker_ratio: f64) -> DAGGenCfg {
    let total_time = 80_000;
    let block_interval = 200;
    let total_blocks = total_time as f64 / block_interval as f64;
    let honest = 1.0 - attacker_ratio;
    let miner_a = honest * 0.6;
    let miner_b = honest * 0.4;

    let hide_count = ((total_blocks * attacker_ratio).round() as usize).clamp(6, 80);
    let release_time_sec = (total_time / 1000 / 2).max(6);

    DAGGenCfg {
        total_time,
        block_interval,
        miners: vec![
            Miner {
                name: "honest_a",
                hash_power_ratio: miner_a,
                network_delay: 80,
                is_attacker: false,
                hide_blocks_plan: vec![],
            },
            Miner {
                name: "honest_b",
                hash_power_ratio: miner_b,
                network_delay: 120,
                is_attacker: false,
                hide_blocks_plan: vec![],
            },
            Miner {
                name: "adv",
                hash_power_ratio: attacker_ratio,
                network_delay: 40,
                is_attacker: true,
                hide_blocks_plan: vec![HideBlocksBurst {
                    start_time_sec: 3,
                    block_count: hide_count,
                    release_time_sec: release_time_sec as u64,
                    release_interval_ms: 15,
                }],
            },
        ],
    }
}
