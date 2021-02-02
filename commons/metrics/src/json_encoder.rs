// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::flatten_metric_with_labels;
use prometheus::{
    proto::{MetricFamily, MetricType},
    Encoder, Result,
};
use std::{collections::HashMap, io::Write};

const JSON_FORMAT: &str = "application/json";

/// An implementation of an [`Encoder`](::Encoder) that converts a `MetricFamily` proto message
/// into `fbagent` json
///
/// This implementation converts metric{dimensions,...} -> value to a flat string with a value.
/// e.g., "requests{method="GET", service="accounts"} -> 8 into
/// requests.GET.account -> 8
/// For now, it ignores timestamps (if set on the metric)
#[derive(Debug, Default)]
pub struct JsonEncoder;

impl Encoder for JsonEncoder {
    fn encode<W: Write>(&self, metric_familys: &[MetricFamily], writer: &mut W) -> Result<()> {
        let mut export_me: HashMap<String, f64> = HashMap::new();

        for mf in metric_familys {
            let name = mf.get_name();
            let metric_type = mf.get_field_type();

            for m in mf.get_metric() {
                match metric_type {
                    MetricType::COUNTER => {
                        export_me.insert(
                            flatten_metric_with_labels(name, m),
                            m.get_counter().get_value(),
                        );
                    }
                    MetricType::GAUGE => {
                        export_me.insert(
                            flatten_metric_with_labels(name, m),
                            m.get_gauge().get_value(),
                        );
                    }
                    MetricType::HISTOGRAM => {
                        // write the sum and counts
                        let h = m.get_histogram();

                        export_me.insert(
                            flatten_metric_with_labels(&format!("{}_count", name), m),
                            h.get_sample_count() as f64,
                        );
                        export_me.insert(
                            flatten_metric_with_labels(&format!("{}_sum", name), m),
                            h.get_sample_sum(),
                        );
                    }
                    _ => {
                        // do nothing; unimplemented
                    }
                }
            }
        }

        writer.write_all(
            serde_json::to_string(&export_me)
                .map_err(|e| prometheus::Error::Msg(e.to_string()))?
                .as_bytes(),
        )?;
        Ok(())
    }

    fn format_type(&self) -> &str {
        JSON_FORMAT
    }
}
