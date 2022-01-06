// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use benchmarks::chain::ChainBencher;
use criterion::BenchmarkId;
#[allow(deprecated)]
use criterion::{criterion_group, criterion_main, Benchmark, Criterion};
#[cfg(target_os = "linux")]
use pprof::criterion::{Output, PProfProfiler};

#[allow(deprecated)]
fn block_apply(c: &mut Criterion) {
    ::logger::init();
    for i in &[10u64, 1000] {
        c.bench(
            "block_apply",
            Benchmark::new(format!("block_apply_{:?}", i), move |b| {
                let bencher = ChainBencher::new(Some(*i));
                bencher.bench(b)
            })
            .sample_size(10),
        );
    }
}

#[allow(deprecated)]
fn query_block(c: &mut Criterion) {
    ::logger::init();
    for block_num in &[10u64, 1000u64] {
        let bencher = ChainBencher::new(Some(*block_num));
        bencher.execute();

        for i in &[100u64, 1000, 10000] {
            let id = format!("query_block_in({:?})_times({:?})", block_num, i,);
            let bencher_local = bencher.clone();
            c.bench(
                "query_block",
                Benchmark::new(id, move |b| bencher_local.query_bench(b, *i)).sample_size(10),
            );
        }
    }
}

fn block_apply_with_create_account(c: &mut Criterion) {
    ::logger::init();
    let mut group = c.benchmark_group("block_apply_with_create_account");
    group.sample_size(10);
    let bench_id = "block_apply_with_create_account";
    for i in &[10u64, 1000] {
        let mut bencher = ChainBencher::new(Some(*i));
        group.bench_function(BenchmarkId::new(bench_id, i), move |b| {
            b.iter(|| bencher.execute_transaction_with_create_account())
        });
    }
}

fn block_apply_with_fixed_account(c: &mut Criterion) {
    ::logger::init();
    let mut group = c.benchmark_group("block_apply_with_fixed_account");
    group.sample_size(10);
    let bench_id = "block_apply_with_fixed_account";
    for i in &[10u64, 1000] {
        let mut bencher = ChainBencher::new(Some(*i));
        group.bench_function(BenchmarkId::new(bench_id, i), move |b| {
            b.iter(|| bencher.execute_transaction_with_fixed_account())
        });
    }
}
#[cfg(target_os = "linux")]
criterion_group!(
    name=starcoin_chain_benches;
    config = Criterion::default()
    .with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets=block_apply,query_block, block_apply_with_create_account,block_apply_with_fixed_account);
#[cfg(not(target_os = "linux"))]
criterion_group!(
    starcoin_chain_benches,
    block_apply,
    query_block,
    block_apply_with_create_account,
    block_apply_with_fixed_account
);

criterion_main!(starcoin_chain_benches);
