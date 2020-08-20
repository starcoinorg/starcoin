// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use benchmarks::chain::ChainBencher;
use criterion::{criterion_group, criterion_main, Benchmark, Criterion};

fn block_apply(c: &mut Criterion) {
    ::logger::init();
    for i in vec![100u64, 500].into_iter() {
        c.bench(
            "block_try_connect",
            Benchmark::new(format!("connect_branch_{:?}", i), move |b| {
                let bencher = ChainBencher::new(Some(i));
                bencher.bench(b, None)
            })
            .sample_size(10),
        );
    }
}

fn query_block(c: &mut Criterion) {
    for i in vec![100u64, 500].into_iter() {
        for j in vec![2u64, 5, 10].into_iter() {
            for k in vec![100u64, 500].into_iter() {
                for id in [format!("query_block_{:?}_{:?}_{:?}", i, j, k)].iter() {
                    c.bench(
                        "query_block",
                        Benchmark::new(id, move |b| {
                            let bencher = ChainBencher::new(Some(i));
                            bencher.execute(Some(j));
                            bencher.query_bench(b, k)
                        })
                        .sample_size(10),
                    );
                }
            }
        }
    }
}

criterion_group!(starcoin_chain_benches, block_apply, query_block);
criterion_main!(starcoin_chain_benches);
