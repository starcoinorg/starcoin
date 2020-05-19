// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
use benchmarks::sync::SyncBencher;
use criterion::{criterion_group, criterion_main, Benchmark, Criterion};

fn full_sync(c: &mut Criterion) {
    for i in vec![10u64, 20, 50].into_iter() {
        c.bench(
            "full_sync",
            Benchmark::new(format!("full_sync_{:?}", i), move |b| {
                let sync_bencher = SyncBencher {};
                sync_bencher.bench_full_sync(b, i)
            })
            .sample_size(10),
        );
    }
}

criterion_group!(starcoin_sync_benches, full_sync);
criterion_main!(starcoin_sync_benches);
