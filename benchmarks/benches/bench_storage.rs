// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
use benchmarks::storage::StorageBencher;
use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use crypto::HashValue;
#[cfg(target_os = "linux")]
use pprof::criterion::{Output, PProfProfiler};
use starcoin_accumulator::{accumulator_info::AccumulatorInfo, Accumulator, MerkleAccumulator};
use starcoin_config::RocksdbConfig;
use starcoin_storage::cache_storage::CacheStorage;
use starcoin_storage::db_storage::DBStorage;
use starcoin_storage::storage::StorageInstance;
use starcoin_storage::Storage;
use std::sync::Arc;

//
// Storage benchmarks
//
fn storage_transaction(c: &mut Criterion) {
    ::logger::init_for_test();
    let path = starcoin_config::temp_dir();
    c.bench_function("storage_transaction", |b| {
        let storage = Storage::new(StorageInstance::new_cache_and_db_instance(
            CacheStorage::new(None),
            DBStorage::new(path.as_ref(), RocksdbConfig::default(), None).unwrap(),
        ))
        .unwrap();
        let bencher = StorageBencher::new(storage);
        bencher.bench(b)
    });
}

/// accumulator benchmarks
fn accumulator_append(c: &mut Criterion) {
    ::logger::init_for_test();
    let path = starcoin_config::temp_dir();
    c.bench_function("accumulator_append", |b| {
        let storage = Arc::new(
            Storage::new(StorageInstance::new_cache_and_db_instance(
                CacheStorage::new(None),
                DBStorage::new(path.as_ref(), RocksdbConfig::default(), None).unwrap(),
            ))
            .unwrap(),
        );
        let leaves = create_leaves(0..100);
        b.iter_batched(
            || {
                MerkleAccumulator::new_with_info(
                    AccumulatorInfo::default(),
                    Arc::new(storage.get_transaction_accumulator_storage()),
                )
            },
            |bench| {
                bench.append(&leaves).unwrap();
                bench.flush().unwrap();
            },
            BatchSize::LargeInput,
        )
    });
}

fn create_leaves(nums: std::ops::Range<usize>) -> Vec<HashValue> {
    nums.map(|x| HashValue::sha3_256_of(x.to_be_bytes().as_ref()))
        .collect()
}
#[cfg(target_os = "linux")]
criterion_group!(
    name=starcoin_storage_benches;
    config = Criterion::default()
    .with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets=storage_transaction, accumulator_append
);
#[cfg(not(target_os = "linux"))]
criterion_group!(
    starcoin_storage_benches,
    storage_transaction,
    accumulator_append
);
criterion_main!(starcoin_storage_benches);
