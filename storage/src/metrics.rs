// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2
use anyhow::Result;
use once_cell::sync::Lazy;
use starcoin_metrics::{
    register_histogram_vec, register_int_counter_vec, HistogramTimer, HistogramVec, IntCounterVec,
};

pub static STORAGE_COUNTERS: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "starcoin_storage",
        "Counters of how many storage read/write",
        &["storage_type", "key_type", "method", "result"]
    )
    .unwrap()
});

pub static STORAGE_TIMES: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "starcoin_storage_time",
        "Histogram of storage",
        &["storage_type", "key_type", "method"]
    )
    .unwrap()
});

pub enum ResultType {
    NONE,
    SOME,
    OK,
    ERROR,
}

impl ResultType {
    pub fn into_str(self) -> &'static str {
        match self {
            ResultType::NONE => "none",
            ResultType::ERROR => "error",
            ResultType::OK => "ok",
            ResultType::SOME => "some",
        }
    }
}

pub trait AsResultType {
    fn as_result_type(&self) -> ResultType;
}

impl AsResultType for Result<()> {
    fn as_result_type(&self) -> ResultType {
        match self {
            Ok(_) => ResultType::OK,
            Err(_) => ResultType::ERROR,
        }
    }
}

impl AsResultType for Result<bool> {
    fn as_result_type(&self) -> ResultType {
        match self {
            Ok(_) => ResultType::OK,
            Err(_) => ResultType::ERROR,
        }
    }
}

impl<T> AsResultType for Result<Option<T>> {
    fn as_result_type(&self) -> ResultType {
        match self {
            Ok(v) => match v {
                Some(_) => ResultType::SOME,
                None => ResultType::NONE,
            },
            Err(_) => ResultType::ERROR,
        }
    }
}

pub struct MetricsRecord<'a> {
    storage_type: &'a str,
    key_type: &'a str,
    method: &'a str,
    timer: HistogramTimer,
}

impl<'a> MetricsRecord<'a> {
    pub fn new(storage_type: &'a str, key_type: &'a str, method: &'a str) -> Self {
        let timer = STORAGE_TIMES
            .with_label_values(&[storage_type, key_type, method])
            .start_timer();
        MetricsRecord {
            storage_type,
            key_type,
            method,
            timer,
        }
    }
    pub fn end<R>(self, result: R) -> R
    where
        R: AsResultType,
    {
        let result_type = result.as_result_type();
        STORAGE_COUNTERS
            .with_label_values(&[
                self.storage_type,
                self.key_type,
                self.method,
                result_type.into_str(),
            ])
            .inc();
        self.timer.stop_and_record();
        result
    }

    pub fn end_with<R, F>(self, f: F) -> R
    where
        F: FnOnce() -> R,
        R: AsResultType,
    {
        let r = f();
        self.end(r)
    }
}

pub fn record_metrics<'a>(
    storage_type: &'a str,
    key_type: &'a str,
    method: &'a str,
) -> MetricsRecord<'a> {
    MetricsRecord::new(storage_type, key_type, method)
}
