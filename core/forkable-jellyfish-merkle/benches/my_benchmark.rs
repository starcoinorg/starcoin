use forkable_jellyfish_merkle::{
    blob::Blob, mock_tree_store::MockTreeStore, nibble::Nibble, JellyfishMerkleTree,
};

use rand::{rngs::StdRng, Rng, SeedableRng};
use starcoin_crypto::hash::*;
use std::collections::HashMap;

use criterion::{black_box, criterion_group, criterion_main, Bencher, BenchmarkId, Criterion};

fn bench_get_1000_keys(c: &mut Criterion) {
    let seed: &[u8] = &[1, 2, 3, 4];
    let (kvs, db, root) = prepare_tree(seed, 1000);
    // c.bench_with_input("bench_1000_keys".into(), &(kvs, db), |b, input| {
    //     b.iter_batched()
    // });
    c.bench_function("bench_1000_keys", |b| {
        // b.iter_batched(|| kvs),
        let tree = JellyfishMerkleTree::new(&db);
        b.iter(|| {
            for (k, v) in &kvs {
                let (value, proof) = tree.get_with_proof(root, *k).unwrap();
                assert_eq!(value.unwrap(), *v);
                // assert!(proof.verify(root, *k, Some(v)).is_ok());
            }
        });
    });
}

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

fn prepare_tree(
    seed: &[u8],
    num_keys: usize,
) -> (HashMap<HashValue, Blob>, MockTreeStore, HashValue) {
    let kvs = gen_kv_from_seed(seed, num_keys);

    let db = MockTreeStore::default();
    let tree = JellyfishMerkleTree::new(&db);

    let kv_vec = kvs.clone().into_iter().collect::<Vec<_>>();
    let (root, batch) = tree.insert_all(None, kv_vec).unwrap();
    db.write_tree_update_batch(batch).unwrap();
    (kvs, db, root)
}

criterion_group!(benches, bench_get_1000_keys);
criterion_main!(benches);
