use std::{
    collections::HashMap,
    error::Error,
    fs::OpenOptions,
    io::Write,
};

use chrono::NaiveDateTime;
use plotters::prelude::*;
use starcoin_crypto::HashValue;

/// Represents the result of a transaction execution event
#[derive(Clone)]
pub enum TransactionExecutionResult {
    Added(String),
    Rejected(String),
    Culled(String),
    Executed(String),
    #[allow(dead_code)]
    ExecutedNotInMain(String),
    Other(String),
}

impl std::fmt::Debug for TransactionExecutionResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionExecutionResult::Added(op_time) => {
                write!(f, "TransactionExecutionResult::Added({})", op_time)
            }
            TransactionExecutionResult::Rejected(op_time) => {
                write!(f, "TransactionExecutionResult::Rejected({})", op_time)
            }
            TransactionExecutionResult::Culled(op_time) => {
                write!(f, "TransactionExecutionResult::Culled({})", op_time)
            }
            TransactionExecutionResult::Executed(op_time) => {
                write!(f, "TransactionExecutionResult::Executed({})", op_time)
            }
            TransactionExecutionResult::ExecutedNotInMain(op_time) => write!(
                f,
                "TransactionExecutionResult::ExecutedNotInMain({})",
                op_time
            ),
            TransactionExecutionResult::Other(op_time) => {
                write!(f, "TransactionExecutionResult::Other({})", op_time)
            }
        }
    }
}

/// Results dumper for transaction execution data
pub struct ResultsDumper<'a> {
    transaction_data: &'a HashMap<HashValue, Vec<TransactionExecutionResult>>,
}

impl<'a> ResultsDumper<'a> {
    pub fn new(transaction_data: &'a HashMap<HashValue, Vec<TransactionExecutionResult>>) -> Self {
        Self { transaction_data }
    }

    /// Dump results to text file and SVG chart
    pub fn dump_results(&self) -> anyhow::Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open("./transaction_results.txt")?;

        for (transaction, results) in self.transaction_data {
            writeln!(
                file,
                "transaction id: {}, results: {:?}",
                *transaction, results
            )?;
        }

        match self.export_latency_timeline_svg("./latency_timeline.svg") {
            Ok(_) => (),
            Err(e) => {
                return Err(anyhow::format_err!(
                    "failed to export latency timeline svg: {}",
                    e
                ));
            }
        }
        Ok(())
    }

    fn collect_executions(&self) -> (HashMap<String, Vec<f64>>, usize, usize) {
        let fmt = "%Y-%m-%d %H:%M:%S%.3f";
        let mut grouped: HashMap<String, Vec<f64>> = HashMap::new();
        let mut unique_txn_count = 0usize;
        let mut duplicate_exec_count = 0usize;

        for events in self.transaction_data.values() {
            let mut added_times = Vec::new();
            let mut executed_times = Vec::new();

            for ev in events {
                match ev {
                    TransactionExecutionResult::Added(ts) => {
                        if let Ok(t) = NaiveDateTime::parse_from_str(ts, fmt) {
                            added_times.push((ts.clone(), t));
                        }
                    }
                    TransactionExecutionResult::Executed(ts)
                    | TransactionExecutionResult::ExecutedNotInMain(ts) => {
                        if let Ok(t) = NaiveDateTime::parse_from_str(ts, fmt) {
                            executed_times.push(t);
                        }
                    }
                    _ => {}
                }
            }

            if added_times.is_empty() {
                continue;
            }

            unique_txn_count += 1;

            let (first_add_str, first_add) = &added_times[0];

            if executed_times.is_empty() {
                grouped
                    .entry(first_add_str.clone())
                    .or_default()
                    .push(f64::INFINITY);
                continue;
            }

            if executed_times.len() > 1 {
                duplicate_exec_count += executed_times.len() - 1;
            }

            let last_exec = executed_times.iter().max().unwrap();
            let delay = *last_exec - *first_add;
            if let Some(us) = delay.num_microseconds() {
                let ms = us as f64 / 1000.0;
                if ms >= 0.0 {
                    grouped.entry(first_add_str.clone()).or_default().push(ms);
                }
            }
        }

        (grouped, unique_txn_count, duplicate_exec_count)
    }

    pub fn export_latency_timeline_svg(&self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let (grouped, unique_txn_count, duplicate_exec_count) = self.collect_executions();
        if grouped.is_empty() {
            return Ok(());
        }

        let fmt = "%Y-%m-%d %H:%M:%S%.3f";
        let mut sorted_times: Vec<String> = grouped.keys().cloned().collect();
        sorted_times.sort_by(|a, b| {
            let t1 = NaiveDateTime::parse_from_str(a, fmt).ok();
            let t2 = NaiveDateTime::parse_from_str(b, fmt).ok();
            match (t1, t2) {
                (Some(t1), Some(t2)) => t1.cmp(&t2),
                _ => std::cmp::Ordering::Equal,
            }
        });

        let mut latency_counts: Vec<(String, HashMap<i64, usize>)> = Vec::new();
        for time_str in &sorted_times {
            let delays = grouped.get(time_str).unwrap();
            let mut count_map: HashMap<i64, usize> = HashMap::new();
            for d in delays {
                if d.is_finite() {
                    let key = (*d * 10.0).round() as i64;
                    *count_map.entry(key).or_insert(0) += 1;
                }
            }
            latency_counts.push((time_str.clone(), count_map));
        }

        let max_latency: f64 = grouped
            .values()
            .flat_map(|v| v.iter())
            .filter(|d| d.is_finite())
            .fold(0.0f64, |acc, &d| acc.max(d))
            .max(1.0);

        let root = SVGBackend::new(file_path, (1600, 800)).into_drawing_area();
        root.fill(&WHITE)?;

        let num_bars = sorted_times.len();

        let mut chart = ChartBuilder::on(&root)
            .caption(
                "Transaction Latency (Added to Executed)",
                ("sans-serif", 28),
            )
            .margin(20)
            .x_label_area_size(120)
            .y_label_area_size(70)
            .build_cartesian_2d(0f64..(num_bars as f64), 0f64..max_latency)?;

        chart
            .configure_mesh()
            .x_desc("Added Time")
            .y_desc("Latency (ms)")
            .x_label_formatter(&|x| {
                let idx = *x as usize;
                if idx < sorted_times.len() {
                    let t = &sorted_times[idx];
                    if let Some(pos) = t.find(' ') {
                        t[pos + 1..].to_string()
                    } else {
                        t.clone()
                    }
                } else {
                    String::new()
                }
            })
            .axis_desc_style(("sans-serif", 20))
            .label_style(("sans-serif", 10))
            .x_labels(num_bars.min(15))
            .draw()?;

        let bar_width = 0.8;
        for (idx, (time_str, count_map)) in latency_counts.iter().enumerate() {
            let delays = grouped.get(time_str).unwrap();
            let max_delay = delays
                .iter()
                .filter(|d| d.is_finite())
                .fold(0.0f64, |acc, &d| acc.max(d));

            if max_delay > 0.0 {
                let x_center = idx as f64 + 0.5;
                let x_left = x_center - bar_width / 2.0;
                let x_right = x_center + bar_width / 2.0;

                chart.draw_series(std::iter::once(Rectangle::new(
                    [(x_left, 0.0), (x_right, max_delay.min(max_latency))],
                    RGBColor(50, 100, 220).filled(),
                )))?;

                for (&latency_key, &count) in count_map {
                    let latency = latency_key as f64 / 10.0;
                    if latency > 0.0 && latency < max_delay {
                        chart.draw_series(std::iter::once(PathElement::new(
                            vec![(x_left, latency), (x_right, latency)],
                            ShapeStyle::from(&WHITE).stroke_width(2),
                        )))?;

                        chart.draw_series(std::iter::once(Text::new(
                            format!("{}", count),
                            (x_center, latency + max_latency * 0.01),
                            ("sans-serif", 10).into_font().color(&WHITE),
                        )))?;
                    }
                }

                let total_count: usize = count_map.values().sum();
                chart.draw_series(std::iter::once(Text::new(
                    format!("{}", total_count),
                    (x_center, max_delay + max_latency * 0.02),
                    ("sans-serif", 12).into_font().color(&BLACK),
                )))?;
            }
        }

        let all_delays: Vec<f64> = grouped
            .values()
            .flat_map(|v| v.iter())
            .filter(|d| d.is_finite())
            .copied()
            .collect();

        let total_txns = all_delays.len();
        let min_delay = all_delays.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max_delay_stat = all_delays.iter().fold(0.0f64, |a, &b| a.max(b));
        let avg_delay = if total_txns > 0 {
            all_delays.iter().sum::<f64>() / total_txns as f64
        } else {
            0.0
        };
        let median_delay = if total_txns > 0 {
            let mut sorted = all_delays.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            if sorted.len() % 2 == 0 {
                (sorted[sorted.len() / 2 - 1] + sorted[sorted.len() / 2]) / 2.0
            } else {
                sorted[sorted.len() / 2]
            }
        } else {
            0.0
        };

        let tps = if !sorted_times.is_empty() && sorted_times.len() >= 2 {
            let first_time = NaiveDateTime::parse_from_str(&sorted_times[0], fmt).ok();
            let last_time =
                NaiveDateTime::parse_from_str(&sorted_times[sorted_times.len() - 1], fmt).ok();
            if let (Some(first), Some(last)) = (first_time, last_time) {
                let duration_secs = (last - first).num_milliseconds() as f64 / 1000.0;
                if duration_secs > 0.0 {
                    total_txns as f64 / duration_secs
                } else {
                    total_txns as f64
                }
            } else {
                0.0
            }
        } else {
            total_txns as f64
        };

        let duplicate_pct = if unique_txn_count > 0 {
            duplicate_exec_count as f64 / unique_txn_count as f64 * 100.0
        } else {
            0.0
        };

        let stats_text = format!(
            "TPS: {:.2} | Total Exec: {} | Unique Txn: {} | Dup: {} ({:.1}%) | Min: {:.2}ms | Max: {:.2}ms | Avg: {:.2}ms | Median: {:.2}ms",
            tps, total_txns, unique_txn_count, duplicate_exec_count, duplicate_pct, min_delay, max_delay_stat, avg_delay, median_delay
        );

        root.draw(&Text::new(
            stats_text,
            (800, 780),
            ("sans-serif", 16).into_font().color(&BLACK),
        ))?;

        root.present()?;
        Ok(())
    }
}
