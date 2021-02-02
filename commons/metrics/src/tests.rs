use super::*;
use crate::json_encoder::JsonEncoder;
use prometheus::{
    core::{Collector, Metric},
    IntCounter, IntCounterVec, Opts,
};
use serde_json::Value;

#[test]
fn test_flatten_labels() {
    // generate counters for testing
    let counter = IntCounter::new("counter_1", "Test counter 1").unwrap();
    let res = flatten_metric_with_labels("counter_1", &counter.metric());
    assert_eq!("counter_1", res.as_str());

    let counter = IntCounterVec::new(
        Opts::new("counter_2", "Example counter for testing"),
        &["label_me"],
    )
    .unwrap();
    let res = flatten_metric_with_labels("counter_2", &counter.with_label_values(&[""]).metric());
    assert_eq!("counter_2", res.as_str());

    let res =
        flatten_metric_with_labels("counter_2", &counter.with_label_values(&["hello"]).metric());
    assert_eq!("counter_2.hello", res.as_str());

    let counter = IntCounterVec::new(
        Opts::new("counter_2", "Example counter for testing"),
        &["label_me", "label_me_too"],
    )
    .unwrap();
    let res =
        flatten_metric_with_labels("counter_3", &counter.with_label_values(&["", ""]).metric());
    assert_eq!("counter_3", res.as_str());

    let res = flatten_metric_with_labels(
        "counter_3",
        &counter.with_label_values(&["hello", "world"]).metric(),
    );
    assert_eq!("counter_3.hello.world", res.as_str());
}

#[test]
fn test_encoder() {
    let counter = IntCounterVec::new(
        Opts::new("testing_count", "Test Counter"),
        &["method", "result"],
    )
    .unwrap();
    // add some test data
    counter.with_label_values(&["get", "302"]).inc();
    counter.with_label_values(&["get", "302"]).inc();
    counter.with_label_values(&["get", "404"]).inc();
    counter.with_label_values(&["put", ""]).inc();

    let metric_family = counter.collect();
    let mut data_writer = Vec::<u8>::new();
    let encoder = JsonEncoder;
    let res = encoder.encode(&metric_family, &mut data_writer);
    assert!(res.is_ok());

    let expected: &str = r#"
        {
            "testing_count.get.302": 2.0,
            "testing_count.get.404": 1.0,
            "testing_count.put": 1.0
        }"#;

    let v: Value = serde_json::from_slice(&data_writer).unwrap();
    let expected_v: Value = serde_json::from_str(expected).unwrap();

    assert_eq!(v, expected_v);
}
