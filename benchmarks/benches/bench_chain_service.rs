// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
use actix::System;
use benchmarks::chain_service::ChainServiceBencher;
use criterion::{criterion_group, criterion_main, Benchmark, Criterion};
use starcoin_bus::BusActor;

fn block_try_connect(c: &mut Criterion) {
    ::logger::init();
    for i in vec![100u64, 500].into_iter() {
        c.bench(
            "block_try_connect",
            Benchmark::new(format!("connect_branch_{:?}", i), move |b| {
                let mut system = System::new("chain-bench");
                let fut = async move { BusActor::launch() };
                let bus = system.block_on(fut);
                let bencher = ChainServiceBencher::new(Some(i), bus);
                bencher.bench(b, None)
            })
            .sample_size(10),
        );
    }
}

fn block_chain_branch(c: &mut Criterion) {
    for i in vec![100u64, 500].into_iter() {
        for j in vec![2u64, 5, 10].into_iter() {
            for id in [format!("branches_{:?}_{:?}", i, j)].iter() {
                c.bench(
                    "block_chain_branch",
                    Benchmark::new(id, move |b| {
                        let mut system = System::new("chain-bench");
                        let fut = async move { BusActor::launch() };
                        let bus = system.block_on(fut);
                        let bencher = ChainServiceBencher::new(Some(i), bus);
                        bencher.bench(b, Some(j))
                    })
                    .sample_size(10),
                );
            }
        }
    }
}

fn query_master_block(c: &mut Criterion) {
    for i in vec![100u64, 500].into_iter() {
        for j in vec![2u64, 5, 10].into_iter() {
            for k in vec![100u64, 500].into_iter() {
                for id in [format!("query_block_{:?}_{:?}_{:?}", i, j, k)].iter() {
                    c.bench(
                        "query_master_block",
                        Benchmark::new(id, move |b| {
                            let mut system = System::new("chain-bench");
                            let fut = async move { BusActor::launch() };
                            let bus = system.block_on(fut);
                            let bencher = ChainServiceBencher::new(Some(i), bus);
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

criterion_group!(
    starcoin_chain_service_benches,
    block_try_connect,
    block_chain_branch,
    query_master_block
);
criterion_main!(starcoin_chain_service_benches);
