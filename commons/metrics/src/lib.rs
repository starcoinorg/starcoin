// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![recursion_limit = "128"]
extern crate prometheus;

use anyhow::Result;
use prometheus::core::{
    AtomicU64, GenericCounter, GenericCounterVec, GenericGauge, GenericGaugeVec,
};
use prometheus::{
    core::{Collector, Metric},
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
#[cfg(any(target_os = "linux", target_os = "macos"))]
mod process_collector;
#[macro_use]
pub mod macros;

pub use op_counters::{DurationHistogram, OpMetrics};
// Re-export counter types from prometheus crate
use prometheus::proto::LabelPair;
pub use prometheus::{
    default_registry, histogram_opts, labels, opts, register_counter, register_counter_vec,
    register_gauge, register_gauge_vec, register_histogram, register_histogram_vec,
    register_int_counter, register_int_counter_vec, register_int_gauge, register_int_gauge_vec,
    Error as PrometheusError, Histogram, HistogramOpts, HistogramTimer, HistogramVec, IntCounter,
    IntCounterVec, IntGauge, IntGaugeVec, Opts, Registry,
};

pub mod proto {
    pub use prometheus::proto::*;
}

pub mod prometheus_export {
    pub use prometheus::register;
}

pub type UIntGaugeVec = GenericGaugeVec<AtomicU64>;
pub type UIntGauge = GenericGauge<AtomicU64>;

pub type UIntCounterVec = GenericCounterVec<AtomicU64>;
pub type UIntCounter = GenericCounter<AtomicU64>;

pub fn register<T: Clone + Collector + 'static>(
    metric: T,
    registry: &Registry,
) -> Result<T, PrometheusError> {
    registry.register(Box::new(metric.clone()))?;
    Ok(metric)
}

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

pub fn get_all_metrics(registry: &Registry) -> HashMap<String, String> {
    let all_metric_families = registry.gather();
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

/**
This method takes Prometheus metrics with dimensions (represented as label:value tags)
and converts it into a dot-separated string.

Example:
Prometheus metric: error_count{method: "get_account", error="connection_error"}
Result: error_count.get_account.connection_error

If the set of labels is empty, only the name is returned
Example:
Prometheus metric: errors
Result: errors

This is useful when exporting metric data to flat time series.
*/
fn flatten_metric_with_labels(name: &str, metric: &prometheus::proto::Metric) -> String {
    let res = String::from(name);

    if metric.get_label().is_empty() {
        res
    } else {
        // string-list.join(".")
        let values: Vec<&str> = metric
            .get_label()
            .iter()
            .map(LabelPair::get_value)
            .filter(|&x| !x.is_empty())
            .collect();
        let values = values.join(".");
        if !values.is_empty() {
            format!("{}.{}", res, values)
        } else {
            res
        }
    }
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

pub fn get_metric_from_registry(
    registry: &Registry,
    metric_name: &str,
    label: Option<(&str, &str)>,
) -> Option<Vec<crate::proto::Metric>> {
    registry.gather().into_iter().find_map(|metric_fm| {
        if metric_fm.get_name() == metric_name {
            let metrics = metric_fm.get_metric().to_vec();
            if let Some((label_name, label_value)) = label {
                metrics
                    .into_iter()
                    .find(|metric| {
                        metric.get_label().iter().any(|label_pair| {
                            label_pair.get_name() == label_name
                                && label_pair.get_value() == label_value
                        })
                    })
                    .map(|metric| vec![metric])
            } else {
                Some(metrics)
            }
        } else {
            None
        }
    })
}

#[cfg(test)]
mod tests;
