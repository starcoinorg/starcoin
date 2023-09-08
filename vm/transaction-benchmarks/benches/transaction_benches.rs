// Copyright (c) Starcoin
// SPDX-License-Identifier: Apache-2.0

use criterion::{criterion_group, criterion_main, measurement::Measurement, Criterion};
use pprof::criterion::{Output, PProfProfiler};
use proptest::prelude::*;
use starcoin_language_e2e_tests::account_universe::P2PTransferGen;
use starcoin_transaction_benchmarks::transactions::TransactionBencher;
use std::fs::File;

//
// Transaction benchmarks
//

fn peer_to_peer<M: Measurement + 'static>(c: &mut Criterion<M>) {
    let default_num_accounts = 1_000;
    let default_num_transactions = 10_000;
    c.bench_function("peer_to_peer", |b| {
        let bencher = TransactionBencher::new(
            any_with::<P2PTransferGen>((10_000, 10_000_000)),
            default_num_accounts,
            default_num_transactions,
        );
        bencher.bench(b);
    });

    c.bench_function("peer_to_peer_parallel", |b| {
        let bencher = TransactionBencher::new(
            any_with::<P2PTransferGen>((10_000, 10_000_000)),
            default_num_accounts,
            default_num_transactions,
        );
        bencher.bench_parallel(b);
    });
}

criterion_group!(
    name = txn_benches;
    // config = wall_time_measurement().sample_size(10);
    config = Criterion::default().with_profiler(PProfProfiler::new(10, Output::Flamegraph(None)));
    targets = peer_to_peer
);

criterion_main!(txn_benches);
