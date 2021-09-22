use criterion::{criterion_group, criterion_main, Criterion};
use forkable_jellyfish_merkle::{
    blob::Blob, mock_tree_store::MockTreeStore, HashValueKey, JellyfishMerkleTree, RawKey,
};
use rand::{rngs::StdRng, SeedableRng};
use starcoin_crypto::hash::*;
use std::collections::HashMap;

fn bench_get_with_proof(c: &mut Criterion) {
    let (kvs, db, root) = prepare_tree(&[1, 2, 3, 4], 1000);
    let tree = JellyfishMerkleTree::new(&db);
    let k_len = kvs.len();
    let ks = kvs.keys().collect::<Vec<_>>();
    c.bench_function("get_with_proof", |b| {
        let mut i = 0usize;
        b.iter_with_setup(
            || {
                let k = ks[i % k_len];
                i += 1;
                k
            },
            |k| {
                let (value, _proof) = tree.get_with_proof(root, k.key_hash()).unwrap();
                assert_eq!(&value.unwrap(), kvs.get(k).unwrap())
            },
        );
        // b.iter(|| {
        //     for (k, v) in &kvs {
        //         let (value, proof) = tree.get_with_proof(root, *k).unwrap();
        //         assert_eq!(value.unwrap(), *v);
        //         // assert!(proof.verify(root, *k, Some(v)).is_ok());
        //     }
        // });
    });
}

criterion_group!(benches, bench_get_with_proof);
criterion_main!(benches);

fn gen_kv_from_seed(seed: &[u8], num_keys: usize) -> HashMap<HashValueKey, Blob> {
    assert!(seed.len() < 32);
    let mut actual_seed = [0u8; 32];
    actual_seed[..seed.len()].copy_from_slice(seed);
    let mut rng: StdRng = StdRng::from_seed(actual_seed);
    let mut kvs = HashMap::new();
    for _i in 0..num_keys {
        let key = HashValueKey(HashValue::random_with_rng(&mut rng));
        let value = Blob::from(HashValue::random_with_rng(&mut rng).to_vec());
        kvs.insert(key, value);
    }

    kvs
}

fn prepare_tree(
    seed: &[u8],
    num_keys: usize,
) -> (HashMap<HashValueKey, Blob>, MockTreeStore, HashValue) {
    let kvs = gen_kv_from_seed(seed, num_keys);

    let db = MockTreeStore::default();
    let tree = JellyfishMerkleTree::new(&db);

    let kv_vec = kvs.clone().into_iter().collect::<Vec<_>>();
    let (root, batch) = tree.insert_all(None, kv_vec).unwrap();
    db.write_tree_update_batch(batch).unwrap();
    (kvs, db, root)
}
