// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use num_cpus;
use rayon::prelude::*;
use rayon::ThreadPoolBuilder;
use starcoin_config::NodeConfig;
use starcoin_genesis::Genesis;
use starcoin_logger::{init_with_default_level, LogPattern};
use starcoin_transaction_builder::vm2::build_transfer_from_association;
use starcoin_vm2_vm_types::account_address::AccountAddress as Vm2AccountAddress;
use transaction_pool::Verifier as TxPoolVerifier;
use starcoin_txpool::{
    NonceCache, PoolClient, PoolTransaction, SeqNumberAndGasPrice,
    UnverifiedUserTransaction, VerifiedTransaction, Verifier, VerifierOptions,
};
use starcoin_types::multi_transaction::MultiSignedUserTransaction;
use starcoin_storage::Store;
use starcoin_vm2_vm_types::transaction::Transaction as Transaction2;
use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, Once};

static LOG_INIT: Once = Once::new();

fn build_verifier() -> (
    Verifier<PoolClient, SeqNumberAndGasPrice, VerifiedTransaction>,
    MultiSignedUserTransaction,
) {
    LOG_INIT.call_once(|| {
        let _ = init_with_default_level("warn", Some(LogPattern::WithLine));
    });
    let config = NodeConfig::random_for_test();
    let net = config.net().clone();
    let (storage, storage2, chain_info, ..) =
        Genesis::init_storage_for_test(&net).expect("init storage for test");
    let multi_state = storage
        .get_vm_multi_state(chain_info.head().id())
        .expect("multi state from genesis");

    let pool_client = PoolClient::new(
        multi_state.state_root1(),
        multi_state.state_root2(),
        storage,
        storage2,
        NonceCache::new(128),
        None,
    );

    let verifier = Verifier::new(
        pool_client,
        VerifierOptions::default(),
        Arc::new(AtomicUsize::new(0)),
        None,
    );

    let receiver: Vm2AccountAddress = "0x2".parse().expect("valid vm2 address literal");
    let txn = build_transfer_from_association(
        receiver.into(),
        0,
        10_000,
        net.time_service().now_secs() + 3_600,
        net.chain_id().id().into(),
        net.genesis_config2(),
    );
    let signed = match txn {
        Transaction2::UserTransaction(signed) => signed,
        _ => unreachable!("vm2 transfer should be user txn"),
    };

    (verifier, MultiSignedUserTransaction::from(signed))
}

fn bench_verify_transaction(c: &mut Criterion) {
    let (verifier, base_txn) = build_verifier();

    c.bench_function("txpool_verify_transaction", |b| {
        b.iter_batched(
            || PoolTransaction::Unverified(UnverifiedUserTransaction::from(base_txn.clone())),
            |pool_txn| {
                TxPoolVerifier::verify_transaction(&verifier, pool_txn)
                    .expect("transaction should verify");
            },
            BatchSize::SmallInput,
        )
    });
}

fn bench_verify_transaction_parallel(c: &mut Criterion) {
    let (verifier, base_txn) = build_verifier();
    let verifier = Arc::new(verifier);
    let base_txn = Arc::new(base_txn);
    let thread_counts = [4usize, num_cpus::get().max(2)];

    for &threads in &thread_counts {
        let name = format!("txpool_verify_transaction_par_{}", threads);
        let verifier = Arc::clone(&verifier);
        let base_txn = Arc::clone(&base_txn);
        c.bench_function(&name, move |b| {
            let pool = ThreadPoolBuilder::new()
                .num_threads(threads)
                .build()
                .expect("build rayon pool");
            b.iter(|| {
                let txns: Vec<_> = (0..threads)
                    .map(|_| {
                        PoolTransaction::Unverified(UnverifiedUserTransaction::from(
                            base_txn.as_ref().clone(),
                        ))
                    })
                    .collect();
                pool.install(|| {
                    txns.par_iter().for_each(|tx| {
                        let tx = tx.clone();
                        TxPoolVerifier::verify_transaction(&*verifier, tx)
                            .expect("transaction should verify");
                    });
                });
            });
        });
    }
}

criterion_group!(benches, bench_verify_transaction, bench_verify_transaction_parallel);
criterion_main!(benches);
