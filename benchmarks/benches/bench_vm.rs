// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use executor_benchmark::run_benchmark;
#[cfg(target_os = "linux")]
use pprof::criterion::{Output, PProfProfiler};

pub fn transaction_execution(c: &mut Criterion) {
    ::logger::init();
    let mut group = c.benchmark_group("vm");
    group.sample_size(10);
    let bench_id = "transaction_execution";
    for i in vec![1u64, 5, 10, 20, 50].into_iter() {
        group.bench_function(BenchmarkId::new(bench_id, i), |b| {
            b.iter(|| run_benchmark(20, 1_000_000, i as usize, 1))
        });
    }
}

#[cfg(target_os = "linux")]
criterion_group!(
    name=starcoin_vm_benches;
     config = Criterion::default()
    .with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets=transaction_execution);
#[cfg(not(target_os = "linux"))]
criterion_group!(starcoin_vm_benches, transaction_execution);
criterion_main!(starcoin_vm_benches);
