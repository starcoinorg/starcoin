// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
use benchmarks::storage::StorageBencher;
use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use crypto::HashValue;
use starcoin_accumulator::{
    accumulator_info::AccumulatorInfo, node::AccumulatorStoreType, Accumulator, MerkleAccumulator,
};
use starcoin_storage::cache_storage::CacheStorage;
use starcoin_storage::db_storage::DBStorage;
use starcoin_storage::storage::StorageInstance;
use starcoin_storage::IntoSuper;
use starcoin_storage::Storage;
use std::sync::Arc;

//
// Storage benchmarks
//
fn storage_transaction(c: &mut Criterion) {
    ::logger::init_for_test();
    c.bench_function("storage_transaction", |b| {
        let storage = Storage::new(StorageInstance::new_cache_and_db_instance(
            CacheStorage::new(),
            DBStorage::new(starcoin_config::temp_path().as_ref()),
        ))
        .unwrap();
        let bencher = StorageBencher::new(storage);
        bencher.bench(b)
    });
}

/// accumulator benchmarks
fn accumulator_append(c: &mut Criterion) {
    ::logger::init_for_test();
    c.bench_function("accumulator_append", |b| {
        let storage = Arc::new(
            Storage::new(StorageInstance::new_cache_and_db_instance(
                CacheStorage::new(),
                DBStorage::new(starcoin_config::temp_path().as_ref()),
            ))
            .unwrap(),
        );
        let leaves = create_leaves(0..100);
        b.iter_batched(
            || {
                MerkleAccumulator::new_with_info(
                    AccumulatorInfo::default(),
                    AccumulatorStoreType::Transaction,
                    storage.clone().into_super_arc(),
                )
                .unwrap()
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
criterion_group!(
    starcoin_storage_benches,
    storage_transaction,
    accumulator_append
);
criterion_main!(starcoin_storage_benches);
