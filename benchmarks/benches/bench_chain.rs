// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
use actix::System;
use benchmarks::chain::ChainBencher;
use criterion::{criterion_group, criterion_main, Benchmark, Criterion};
use starcoin_bus::BusActor;

fn block_try_connect(c: &mut Criterion) {
    c.bench(
        "block_try_connect",
        Benchmark::new("connect_branch", |b| {
            let mut system = System::new("test");
            let fut = async move { BusActor::launch() };
            let bus = system.block_on(fut);
            let bencher = ChainBencher::new(Some(1000), bus);
            bencher.bench(b)
        })
        .sample_size(10),
    );
}

criterion_group!(starcoin_chain_benches, block_try_connect);
criterion_main!(starcoin_chain_benches);
