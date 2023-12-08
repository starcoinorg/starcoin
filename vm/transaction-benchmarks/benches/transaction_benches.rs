// Copyright (c) Starcoin
// SPDX-License-Identifier: Apache-2.0

use criterion::{criterion_group, criterion_main, measurement::Measurement, Criterion};
use pprof::criterion::{Output, PProfProfiler};
use proptest::prelude::*;
use starcoin_language_e2e_tests::account_universe::P2PTransferGen;
use starcoin_transaction_benchmarks::transactions::TransactionBencher;

//
// Transaction benchmarks
//

// const DEFAULT_NUM_ACCOUNTS: usize = 1_000;
// const DEFAULT_NUM_TRANSACTIONS: usize = 10_000;

fn peer_to_peer<M: Measurement + 'static>(c: &mut Criterion<M>) {
    c.bench_function("peer_to_peer", |b| {
        let bencher = TransactionBencher::new(
            any_with::<P2PTransferGen>((10_000, 10_000_000)),
            // DEFAULT_NUM_ACCOUNTS,
            // DEFAULT_NUM_TRANSACTIONS,
        );
        bencher.bench(b);
    });
}

fn peer_to_peer_parallel<M: Measurement + 'static>(c: &mut Criterion<M>) {
    c.bench_function("peer_to_peer_parallel", |b| {
        let bencher = TransactionBencher::new(
            any_with::<P2PTransferGen>((10_000, 10_000_000)),
            // DEFAULT_NUM_ACCOUNTS,
            // DEFAULT_NUM_TRANSACTIONS,
        );
        bencher.bench_parallel(b);
    });
}

criterion_group!(
    name = txn_benches;
    // config = wall_time_measurement().sample_size(10);
    config = Criterion::default().with_profiler(PProfProfiler::new(10, Output::Flamegraph(None)));
    targets = peer_to_peer, peer_to_peer_parallel
);

criterion_main!(txn_benches);
