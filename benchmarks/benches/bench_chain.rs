// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
use actix::System;
use benchmarks::chain::ChainBencher;
use criterion::{criterion_group, criterion_main, Benchmark, Criterion};
use starcoin_bus::BusActor;

fn block_try_connect(c: &mut Criterion) {
    for i in vec![100u64, 500].into_iter() {
        c.bench(
            "block_try_connect",
            Benchmark::new(format!("connect_branch_{:?}", i), move |b| {
                let mut system = System::new("chain-bench");
                let fut = async move { BusActor::launch() };
                let bus = system.block_on(fut);
                let bencher = ChainBencher::new(Some(i), bus);
                bencher.bench(b, None)
            })
            .sample_size(10),
        );
    }
}

fn block_chain_branch(c: &mut Criterion) {
    for i in vec![100u64, 500].into_iter() {
        for j in vec![5u64, 10].into_iter() {
            for id in [format!("branches_{:?}_{:?}", i, j)].iter() {
                c.bench(
                    "block_chain_branch",
                    Benchmark::new(id, move |b| {
                        let mut system = System::new("chain-bench");
                        let fut = async move { BusActor::launch() };
                        let bus = system.block_on(fut);
                        let bencher = ChainBencher::new(Some(i), bus);
                        bencher.bench(b, Some(j))
                    })
                    .sample_size(10),
                );
            }
        }
    }
}

criterion_group!(
    starcoin_chain_benches,
    block_try_connect,
    block_chain_branch
);
criterion_main!(starcoin_chain_benches);
