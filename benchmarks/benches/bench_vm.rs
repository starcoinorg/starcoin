// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
use criterion::{criterion_group, criterion_main, Criterion};
use executor_benchmark::run_benchmark;

pub fn transaction_execution(c: &mut Criterion) {
    c.bench_function("transaction_execution", |b| {
        b.iter(|| run_benchmark(20, 1000000, 20, 1))
    });
}

criterion_group!(starcoin_vm_benches, transaction_execution);
criterion_main!(starcoin_vm_benches);
