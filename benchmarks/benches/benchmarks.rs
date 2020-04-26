// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
use benchmarks::storage::StorageBencher;
use criterion::{criterion_group, criterion_main, Criterion};
use std::sync::Arc;
use storage::cache_storage::CacheStorage;
use storage::db_storage::DBStorage;
use storage::storage::StorageInstance;
use storage::Storage;

//
// Storage benchmarks
//
fn storage_transaction(c: &mut Criterion) {
    c.bench_function("storage_transaction", |b| {
        let cache_storage = Arc::new(CacheStorage::new());
        let db_storage = Arc::new(DBStorage::new(std::env::temp_dir()));
        let storage = Storage::new(StorageInstance::new_cache_and_db_instance(
            cache_storage,
            db_storage,
        ))
        .unwrap();
        // let storage =
        //     Storage::new(StorageInstance::new_cache_instance(CacheStorage::new())).unwrap();
        // let storage = Storage::new(StorageInstance::new_db_instance(db_storage)).unwrap();
        let bencher = StorageBencher::new(storage);
        bencher.bench(b)
    });
}

criterion_group!(starcoin_benches, storage_transaction);
criterion_main!(starcoin_benches);
