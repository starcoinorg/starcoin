// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![recursion_limit = "128"]
extern crate prometheus;

use anyhow::Result;
use prometheus::core::{AtomicU64, GenericGauge, GenericGaugeVec};
use prometheus::{
    core::{Collector, Metric},
    proto,
    proto::MetricType,
    Encoder, TextEncoder,
};
use starcoin_logger::prelude::*;
use std::{
    collections::HashMap,
    fs::{create_dir_all, File, OpenOptions},
    hash::BuildHasher,
    io::Write,
    path::Path,
    thread, time,
};

mod json_encoder;
pub mod metric_server;
mod op_counters;
mod process_collector;
#[macro_use]
pub mod macros;

pub use op_counters::{DurationHistogram, OpMetrics};
// Re-export counter types from prometheus crate
pub use prometheus::{
    histogram_opts, labels, opts, register, register_counter, register_counter_vec, register_gauge,
    register_gauge_vec, register_histogram, register_histogram_vec, register_int_counter,
    register_int_counter_vec, register_int_gauge, register_int_gauge_vec, Error as PrometheusError,
    Histogram, HistogramTimer, HistogramVec, IntCounter, IntCounterVec, IntGauge, IntGaugeVec,
    Opts,
};

pub type UIntGaugeVec = GenericGaugeVec<AtomicU64>;
pub type UIntGauge = GenericGauge<AtomicU64>;

fn get_metrics_file<P: AsRef<Path>>(dir_path: &P, file_name: &str) -> File {
    create_dir_all(dir_path).expect("Create metrics dir failed");

    let metrics_file_path = dir_path.as_ref().join(file_name);

    info!("Using metrics file {}", metrics_file_path.display());

    OpenOptions::new()
        .append(true)
        .create(true)
        .open(metrics_file_path)
        .expect("Open metrics file failed")
}

fn get_all_metrics_as_serialized_string() -> Result<Vec<u8>> {
    let all_metrics = prometheus::gather();

    let encoder = TextEncoder::new();
    let mut buffer = Vec::new();
    encoder.encode(&all_metrics, &mut buffer)?;
    Ok(buffer)
}

pub fn get_all_metrics() -> HashMap<String, String> {
    let all_metric_families = prometheus::gather();
    let mut all_metrics = HashMap::new();
    for metric_family in all_metric_families {
        let name = metric_family.get_name();
        let metric_type = metric_family.get_field_type();
        for m in metric_family.get_metric() {
            match metric_type {
                MetricType::COUNTER => {
                    all_metrics.insert(
                        flatten_metric_with_labels(name, m),
                        m.get_counter().get_value().to_string(),
                    );
                }
                MetricType::GAUGE => {
                    all_metrics.insert(
                        flatten_metric_with_labels(name, m),
                        m.get_gauge().get_value().to_string(),
                    );
                }
                MetricType::HISTOGRAM => {
                    let h = m.get_histogram();
                    let count = h.get_sample_count();
                    all_metrics.insert(
                        flatten_metric_with_labels(&format!("{}_count", name), m),
                        count.to_string(),
                    );
                    let sum = h.get_sample_sum();
                    all_metrics.insert(
                        flatten_metric_with_labels(&format!("{}_sum", name), m),
                        sum.to_string(),
                    );
                    if count > 0 {
                        let average = sum / (count as f64);
                        all_metrics.insert(
                            flatten_metric_with_labels(&format!("{}_average", name), m),
                            average.to_string(),
                        );
                    }
                }
                MetricType::SUMMARY => panic!("Unsupported Metric 'SUMMARY'"),
                MetricType::UNTYPED => panic!("Unsupported Metric 'UNTYPED'"),
            }
        }
    }

    all_metrics
}

fn flatten_metric_with_labels(name: &str, metric: &proto::Metric) -> String {
    if metric.get_label().is_empty() {
        return name.to_string();
    }
    let label_strings = metric
        .get_label()
        .iter()
        .map(|l| format!("{}={}", l.get_name(), l.get_value()))
        .collect::<Vec<_>>();
    format!("{}{{{}}}", name, label_strings.join(","))
}

// Launches a background thread which will periodically collect metrics
// every interval and write them to the provided file
pub fn dump_all_metrics_to_file_periodically<P: AsRef<Path>>(
    dir_path: &P,
    file_name: &str,
    interval: u64,
) {
    let mut file = get_metrics_file(dir_path, file_name);
    thread::spawn(move || loop {
        let mut buffer = get_all_metrics_as_serialized_string().expect("Error gathering metrics");
        if !buffer.is_empty() {
            buffer.push(b'\n');
            file.write_all(&buffer).expect("Error writing metrics");
        }
        thread::sleep(time::Duration::from_millis(interval));
    });
}

pub fn export_counter<M, S>(col: &mut HashMap<String, String, S>, counter: &M)
where
    M: Metric,
    S: BuildHasher,
{
    let c = counter.metric();
    col.insert(
        c.get_label()[0].get_name().to_string(),
        c.get_counter().get_value().to_string(),
    );
}

pub fn get_metric_name<M>(metric: &M) -> String
where
    M: Collector,
{
    metric.collect()[0].get_name().to_string()
}
