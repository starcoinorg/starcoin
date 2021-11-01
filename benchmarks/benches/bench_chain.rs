// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use benchmarks::chain::ChainBencher;
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
#[cfg(target_os = "linux")]
criterion_group!(
    name=starcoin_chain_benches;
    config = Criterion::default()
    .with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets=block_apply,query_block);
#[cfg(not(target_os = "linux"))]
criterion_group!(starcoin_chain_benches, block_apply, query_block);
criterion_main!(starcoin_chain_benches);
