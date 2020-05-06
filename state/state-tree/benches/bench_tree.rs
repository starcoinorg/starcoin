use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use forkable_jellyfish_merkle::blob::Blob;
use rand::{rngs::StdRng, SeedableRng};
use starcoin_crypto::hash::*;
use starcoin_state_tree::StateTree;
use starcoin_storage::db_storage::DBStorage;
use starcoin_storage::storage::StorageInstance;
use starcoin_storage::Storage;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

fn bench_get_with_proof(c: &mut Criterion) {
    let tmp_dir = tempfile::tempdir().unwrap();
    let store = new_empty_store(tmp_dir.path());
    let tree = StateTree::new(store, None);
    let (kvs, _root) = prepare_tree(&tree, &[1, 2, 3, 4], 100000);
    c.bench_function("get_with_proof", |b| {
        let ks = kvs.keys().collect::<Vec<_>>();
        let k_len = ks.len();
        let mut i = 0usize;

        b.iter_with_setup(
            || {
                let k = ks[i % k_len];
                i = i + 1;
                k
            },
            |k| {
                let (value, _proof) = tree.get_with_proof(k).unwrap();
                assert_eq!(value.unwrap().as_slice(), kvs.get(k).unwrap().as_ref())
            },
        );
    });
}

fn bench_put_and_commit(c: &mut Criterion) {
    let tmp_dir = tempfile::tempdir().unwrap();
    let store = new_empty_store(tmp_dir.path());
    let tree = StateTree::new(store, None);
    let mut rng: StdRng = {
        let seed = [1u8, 2, 3, 4];
        let mut actual_seed = [0u8; 32];
        actual_seed[..seed.len()].copy_from_slice(&seed);
        StdRng::from_seed(actual_seed)
    };
    let mut group = c.benchmark_group("put_and_commit");
    for i in [1u64, 5, 10, 50, 100].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(i), i, |b, n| {
            b.iter_with_setup(
                || {
                    std::iter::repeat(0u8)
                        .take(*n as usize)
                        .map(|_| {
                            let key = HashValue::random_with_rng(&mut rng);
                            let value = Blob::from(HashValue::random_with_rng(&mut rng).to_vec());
                            (key, value)
                        })
                        .collect::<Vec<_>>()
                },
                |kvs| {
                    for (k, v) in kvs {
                        tree.put(k, v.into());
                    }
                    tree.commit().unwrap();
                },
            )
        });
    }
    group.finish();
}

criterion_group!(benches, bench_get_with_proof, bench_put_and_commit);
criterion_main!(benches);

fn gen_kv_from_seed(seed: &[u8], num_keys: usize) -> HashMap<HashValue, Blob> {
    assert!(seed.len() < 32);
    let mut actual_seed = [0u8; 32];
    actual_seed[..seed.len()].copy_from_slice(&seed);
    let mut rng: StdRng = StdRng::from_seed(actual_seed);
    let mut kvs = HashMap::new();
    for _i in 0..num_keys {
        let key = HashValue::random_with_rng(&mut rng);
        let value = Blob::from(HashValue::random_with_rng(&mut rng).to_vec());
        kvs.insert(key, value);
    }

    kvs
}

fn new_empty_store<P: AsRef<Path> + Clone>(p: P) -> Arc<Storage> {
    let db_storage = DBStorage::new(p);
    let store = Storage::new(StorageInstance::new_db_instance(Arc::new(db_storage))).unwrap();
    let store = Arc::new(store);
    store
}

fn prepare_tree(
    state_tree: &StateTree,
    seed: &[u8],
    num_keys: usize,
) -> (HashMap<HashValue, Blob>, HashValue) {
    let kvs = gen_kv_from_seed(seed, num_keys);
    for (k, v) in kvs.clone() {
        state_tree.put(k, v.into());
    }
    let new_root = state_tree.commit().unwrap();
    state_tree.flush().unwrap();

    (kvs, new_root)
}
