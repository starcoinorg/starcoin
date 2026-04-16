// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! CLI binary that drives the DAG parameter sweep via simnet.
//!
//! Usage:
//!   cargo run --release -p simnet --bin sweep_dag_params -- [OPTIONS]
//!
//! The tool iterates over a grid of (K, max_parents, block_interval, network_delay)
//! and outputs CSV rows with throughput and safety metrics.

use anyhow::Result;
use clap::Parser;
use simnet::scene::sweep::{self, SweepParams, SweepResult};
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(
    name = "sweep_dag_params",
    about = "Sweep DAG consensus parameters and measure throughput / safety trade-offs via simnet simulation."
)]
struct Cli {
    /// Comma-separated K values (GhostDAG parameter).
    #[arg(long, default_value = "8,16,32", value_delimiter = ',')]
    k: Vec<u16>,

    /// Comma-separated max_parents values.
    #[arg(long, default_value = "5,10,20", value_delimiter = ',')]
    max_parents: Vec<usize>,

    /// Comma-separated block interval values in milliseconds.
    #[arg(long, default_value = "500,1000,2000", value_delimiter = ',')]
    block_interval: Vec<u64>,

    /// Comma-separated network delay values in milliseconds.
    #[arg(long, default_value = "100,200,500", value_delimiter = ',')]
    network_delay: Vec<u64>,

    /// Number of honest miners in the simulation.
    #[arg(long, default_value = "3")]
    miner_count: usize,

    /// Simulation horizon in milliseconds.
    #[arg(long, default_value = "60000")]
    total_time: u64,

    /// Comma-separated PRNG seeds for averaging (more seeds = more stable results).
    #[arg(long, default_value = "1,2,3", value_delimiter = ',')]
    seeds: Vec<u64>,

    /// Output CSV file path. If omitted, prints to stdout.
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Use the built-in default grid instead of CLI-specified values.
    #[arg(long)]
    default_grid: bool,

    /// Print results to stderr as they complete (always on).
    #[arg(long, default_value = "true")]
    progress: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let grid = if cli.default_grid {
        sweep::default_grid()
    } else {
        build_grid(&cli)
    };

    eprintln!(
        "[sweep] Parameter grid: {} combinations × {} seeds = {} total runs",
        grid.len(),
        cli.seeds.len(),
        grid.len() * cli.seeds.len()
    );

    let results = sweep::run_sweep(&grid, &cli.seeds)?;

    // Output
    let mut writer: Box<dyn Write> = if let Some(ref path) = cli.output {
        Box::new(std::io::BufWriter::new(std::fs::File::create(path)?))
    } else {
        Box::new(std::io::stdout().lock())
    };

    writeln!(writer, "{}", SweepResult::csv_header())?;
    for r in &results {
        writeln!(writer, "{}", r.to_csv_row())?;
    }
    writer.flush()?;

    if let Some(ref path) = cli.output {
        eprintln!("[sweep] Results written to {}", path.display());
    }

    // Print summary
    print_summary(&results);

    Ok(())
}

fn build_grid(cli: &Cli) -> Vec<SweepParams> {
    let mut grid = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for &k in &cli.k {
        for &parents in &cli.max_parents {
            // BlockDAG requires K >= max_parents; clamp and deduplicate
            let parents = parents.min(k as usize);
            for &interval in &cli.block_interval {
                for &delay in &cli.network_delay {
                    let key = (k, parents, interval, delay);
                    if !seen.insert(key) {
                        continue; // skip duplicate after clamping
                    }
                    grid.push(SweepParams {
                        k,
                        max_parents: parents,
                        block_interval_ms: interval,
                        network_delay_ms: delay,
                        total_time_ms: cli.total_time,
                        miner_count: cli.miner_count,
                    });
                }
            }
        }
    }
    grid
}

fn print_summary(results: &[SweepResult]) {
    if results.is_empty() {
        return;
    }

    eprintln!("\n=== SWEEP SUMMARY ===");

    // Best throughput (blocks/s)
    if let Some(best_throughput) = results.iter().max_by(|a, b| {
        a.blocks_per_second
            .partial_cmp(&b.blocks_per_second)
            .unwrap_or(std::cmp::Ordering::Equal)
    }) {
        eprintln!(
            "  Highest blk/s: {:.1} (K={}, parents={}, interval={}ms, delay={}ms)",
            best_throughput.blocks_per_second,
            best_throughput.params.k,
            best_throughput.params.max_parents,
            best_throughput.params.block_interval_ms,
            best_throughput.params.network_delay_ms,
        );
    }

    // Lowest red rate (among configs with >0 blocks)
    if let Some(safest) = results
        .iter()
        .filter(|r| r.total_blocks > 0)
        .min_by(|a, b| {
            a.red_rate
                .partial_cmp(&b.red_rate)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    {
        eprintln!(
            "  Lowest red rate: {:.4} (K={}, parents={}, interval={}ms, delay={}ms)",
            safest.red_rate,
            safest.params.k,
            safest.params.max_parents,
            safest.params.block_interval_ms,
            safest.params.network_delay_ms,
        );
    }

    // Best "efficiency" = blocks_per_second × (1 - red_rate)
    if let Some(best_eff) = results
        .iter()
        .filter(|r| r.total_blocks > 0)
        .max_by(|a, b| {
            let ea = a.blocks_per_second * (1.0 - a.red_rate);
            let eb = b.blocks_per_second * (1.0 - b.red_rate);
            ea.partial_cmp(&eb).unwrap_or(std::cmp::Ordering::Equal)
        })
    {
        let eff = best_eff.blocks_per_second * (1.0 - best_eff.red_rate);
        eprintln!(
            "  Best effective blk/s: {:.1} (K={}, parents={}, interval={}ms, delay={}ms, red_rate={:.4})",
            eff,
            best_eff.params.k,
            best_eff.params.max_parents,
            best_eff.params.block_interval_ms,
            best_eff.params.network_delay_ms,
            best_eff.red_rate,
        );
    }

    // Fastest GhostDAG commit
    if let Some(fastest) = results
        .iter()
        .filter(|r| r.total_blocks > 0)
        .min_by(|a, b| {
            a.avg_commit_ms
                .partial_cmp(&b.avg_commit_ms)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    {
        eprintln!(
            "  Fastest avg commit: {:.3}ms (K={}, parents={}, interval={}ms)",
            fastest.avg_commit_ms,
            fastest.params.k,
            fastest.params.max_parents,
            fastest.params.block_interval_ms,
        );
    }
}
