use starcoin_accumulator::{accumulator_info::AccumulatorInfo, Accumulator, MerkleAccumulator};
use starcoin_config::RocksdbConfig;
use starcoin_crypto::HashValue;

use crate::{
    cache_storage::CacheStorage, db_storage::DBStorage, flexi_dag::SyncFlexiDagSnapshot,
    storage::StorageInstance, Storage, Store, SyncFlexiDagStore,
};
use anyhow::{Ok, Result};

trait SyncFlexiDagManager {
    fn insert_hashes(&self, hashes: Vec<HashValue>) -> Result<HashValue>;
    fn query_by_hash(&self, hash: HashValue) -> Result<Option<SyncFlexiDagSnapshot>>;
    fn fork(&mut self, accumulator_info: AccumulatorInfo) -> Result<()>;
    fn get_hash_by_position(&self, position: u64) -> Result<Option<HashValue>>;
    fn get_accumulator_info(&self) -> AccumulatorInfo;
}

struct SyncFlexiDagManagerImp {
    flexi_dag_storage: Box<dyn SyncFlexiDagStore>,
    accumulator: MerkleAccumulator,
}

impl SyncFlexiDagManagerImp {
    pub fn new() -> Self {
        let flexi_dag_storage = Storage::new(StorageInstance::new_cache_and_db_instance(
            CacheStorage::default(),
            DBStorage::new(
                starcoin_config::temp_dir().as_ref(),
                RocksdbConfig::default(),
                None,
            )
            .unwrap(),
        ))
        .unwrap();
        let accumulator = MerkleAccumulator::new_empty(
            flexi_dag_storage
                .get_accumulator_store(starcoin_accumulator::node::AccumulatorStoreType::SyncDag),
        );
        SyncFlexiDagManagerImp {
            flexi_dag_storage: Box::new(flexi_dag_storage),
            accumulator,
        }
    }

    fn hash_for_hashes(mut hashes: Vec<HashValue>) -> HashValue {
        hashes.sort();
        HashValue::sha3_256_of(&hashes.into_iter().fold([].to_vec(), |mut collect, hash| {
            collect.extend(hash.into_iter());
            collect
        }))
    }
}

// todo: fix this 
impl SyncFlexiDagManager for SyncFlexiDagManagerImp {
    fn insert_hashes(&self, mut child_hashes: Vec<HashValue>) -> Result<HashValue> {
        child_hashes.sort();
        let accumulator_key = Self::hash_for_hashes(child_hashes.clone());
        self.accumulator.append(&[accumulator_key])?;
        self.flexi_dag_storage.put_hashes(
            accumulator_key,
            SyncFlexiDagSnapshot {
                child_hashes,
                accumulator_info: self.get_accumulator_info(),
                k_total_difficulties: None,
                head_block_id: accumulator_key,
            },
        )?;
        Ok(accumulator_key)
    }

    fn query_by_hash(&self, hash: HashValue) -> Result<Option<SyncFlexiDagSnapshot>> {
        self.flexi_dag_storage.query_by_hash(hash)
    }

    fn fork(&mut self, accumulator_info: AccumulatorInfo) -> Result<()> {
        self.accumulator = self.accumulator.fork(Some(accumulator_info));
        Ok(())
    }

    fn get_hash_by_position(&self, position: u64) -> Result<Option<HashValue>> {
        self.accumulator.get_leaf(position)
    }

    fn get_accumulator_info(&self) -> AccumulatorInfo {
        self.accumulator.get_info()
    }
}

#[test]
fn test_syn_dag_accumulator_insert_and_find() {
    let syn_accumulator = SyncFlexiDagManagerImp::new();
    let genesis = HashValue::sha3_256_of(b"genesis");
    let b = HashValue::sha3_256_of(b"b");
    let c = HashValue::sha3_256_of(b"c");
    let d = HashValue::sha3_256_of(b"d");
    let e = HashValue::sha3_256_of(b"e");
    let f = HashValue::sha3_256_of(b"f");
    let h = HashValue::sha3_256_of(b"h");
    let i = HashValue::sha3_256_of(b"i");
    let j = HashValue::sha3_256_of(b"j");
    let k = HashValue::sha3_256_of(b"k");
    let l = HashValue::sha3_256_of(b"l");
    let m = HashValue::sha3_256_of(b"m");

    let genesis_key = syn_accumulator.insert_hashes([genesis].to_vec()).unwrap();
    let layer1 = syn_accumulator
        .insert_hashes([b, c, d, e].to_vec())
        .unwrap();
    let layer2 = syn_accumulator
        .insert_hashes([f, h, i, k].to_vec())
        .unwrap();
    let layer3 = syn_accumulator
        .insert_hashes([j, m, k, l].to_vec())
        .unwrap();
    let layer4 = syn_accumulator.insert_hashes([j, m, l].to_vec()).unwrap();

    assert_eq!(5, syn_accumulator.get_accumulator_info().get_num_leaves());

    assert_eq!(
        genesis_key,
        syn_accumulator.get_hash_by_position(0).unwrap().unwrap()
    );
    assert_eq!(
        layer1,
        syn_accumulator.get_hash_by_position(1).unwrap().unwrap()
    );
    assert_eq!(
        layer2,
        syn_accumulator.get_hash_by_position(2).unwrap().unwrap()
    );
    assert_eq!(
        layer3,
        syn_accumulator.get_hash_by_position(3).unwrap().unwrap()
    );
    assert_eq!(
        layer4,
        syn_accumulator.get_hash_by_position(4).unwrap().unwrap()
    );

    assert_eq!(
        [genesis].to_vec(),
        syn_accumulator
            .query_by_hash(syn_accumulator.get_hash_by_position(0).unwrap().unwrap())
            .unwrap()
            .unwrap()
            .child_hashes
    );
    assert_eq!(
        {
            let mut v = [b, c, d, e].to_vec();
            v.sort();
            v
        },
        syn_accumulator
            .query_by_hash(syn_accumulator.get_hash_by_position(1).unwrap().unwrap())
            .unwrap()
            .unwrap()
            .child_hashes
    );
    assert_eq!(
        {
            let mut v = [f, h, i, k].to_vec();
            v.sort();
            v
        },
        syn_accumulator
            .query_by_hash(syn_accumulator.get_hash_by_position(2).unwrap().unwrap())
            .unwrap()
            .unwrap()
            .child_hashes
    );
    assert_eq!(
        {
            let mut v = [j, m, k, l].to_vec();
            v.sort();
            v
        },
        syn_accumulator
            .query_by_hash(syn_accumulator.get_hash_by_position(3).unwrap().unwrap())
            .unwrap()
            .unwrap()
            .child_hashes
    );
    assert_eq!(
        {
            let mut v = [j, m, l].to_vec();
            v.sort();
            v
        },
        syn_accumulator
            .query_by_hash(syn_accumulator.get_hash_by_position(4).unwrap().unwrap())
            .unwrap()
            .unwrap()
            .child_hashes
    );
}

#[ignore = "todo to use a new test"]
#[test]
fn test_syn_dag_accumulator_fork() {
    let mut syn_accumulator = SyncFlexiDagManagerImp::new();
    let syn_accumulator_target = SyncFlexiDagManagerImp::new();

    let genesis = HashValue::sha3_256_of(b"genesis");
    let b = HashValue::sha3_256_of(b"b");
    let c = HashValue::sha3_256_of(b"c");
    let d = HashValue::sha3_256_of(b"d");
    let e = HashValue::sha3_256_of(b"e");
    let f = HashValue::sha3_256_of(b"f");
    let h = HashValue::sha3_256_of(b"h");
    let i = HashValue::sha3_256_of(b"i");
    let j = HashValue::sha3_256_of(b"j");
    let k = HashValue::sha3_256_of(b"k");
    let l = HashValue::sha3_256_of(b"l");
    let m = HashValue::sha3_256_of(b"m");
    let p = HashValue::sha3_256_of(b"p");
    let v = HashValue::sha3_256_of(b"v");

    let _genesis_key = syn_accumulator.insert_hashes([genesis].to_vec()).unwrap();
    let _genesis_key = syn_accumulator_target
        .insert_hashes([genesis].to_vec())
        .unwrap();

    let layer1 = syn_accumulator
        .insert_hashes([b, c, d, e].to_vec())
        .unwrap();
    let layer2 = syn_accumulator
        .insert_hashes([f, h, i, k].to_vec())
        .unwrap();
    let layer3 = syn_accumulator
        .insert_hashes([j, m, k, l].to_vec())
        .unwrap();
    let layer4 = syn_accumulator.insert_hashes([j, m, l].to_vec()).unwrap();

    let target_layer1 = syn_accumulator_target
        .insert_hashes([b, c, d, e].to_vec())
        .unwrap();
    let target_layer2 = syn_accumulator_target
        .insert_hashes([f, h, i, k].to_vec())
        .unwrap();
    let target_layer3 = syn_accumulator_target
        .insert_hashes([j, m, k, l].to_vec())
        .unwrap();
    let target_layer4 = syn_accumulator_target
        .insert_hashes([p, m, v].to_vec())
        .unwrap();
    let target_layer5 = syn_accumulator_target
        .insert_hashes([p, v].to_vec())
        .unwrap();

    assert_eq!(layer1, target_layer1);
    assert_eq!(layer2, target_layer2);
    assert_eq!(layer3, target_layer3);

    assert_ne!(layer4, target_layer4);
    assert_ne!(
        syn_accumulator.get_accumulator_info().get_num_leaves(),
        syn_accumulator_target
            .get_accumulator_info()
            .get_num_leaves()
    );
    assert_ne!(
        syn_accumulator.get_accumulator_info(),
        syn_accumulator_target.get_accumulator_info()
    );

    let info = syn_accumulator_target
        .query_by_hash(layer3)
        .unwrap()
        .unwrap()
        .accumulator_info;

    println!("{:?}", info);
    assert_eq!(
        layer3,
        syn_accumulator.get_hash_by_position(3).unwrap().unwrap()
    );

    syn_accumulator.fork(info).unwrap();

    assert_eq!(
        layer3,
        syn_accumulator.get_hash_by_position(3).unwrap().unwrap()
    );

    let new_layer4 = syn_accumulator.insert_hashes([p, m, v].to_vec()).unwrap();
    let new_layer5 = syn_accumulator.insert_hashes([p, v].to_vec()).unwrap();

    assert_eq!(new_layer4, target_layer4);
    assert_eq!(new_layer5, target_layer5);
    assert_eq!(
        syn_accumulator.get_accumulator_info().get_num_leaves(),
        syn_accumulator_target
            .get_accumulator_info()
            .get_num_leaves()
    );
    assert_eq!(
        syn_accumulator.get_accumulator_info(),
        syn_accumulator_target.get_accumulator_info()
    );
}

#[test]
fn test_accumulator_temp() {
    let flexi_dag_storage = Storage::new(StorageInstance::new_cache_and_db_instance(
        CacheStorage::default(),
        DBStorage::new(
            starcoin_config::temp_dir().as_ref(),
            RocksdbConfig::default(),
            None,
        )
        .unwrap(),
    ))
    .unwrap();
    let mut accumulator = MerkleAccumulator::new_empty(
        flexi_dag_storage
            .get_accumulator_store(starcoin_accumulator::node::AccumulatorStoreType::SyncDag),
    );
    let _hash1 = accumulator.append(&[HashValue::sha3_256_of(b"a")]).unwrap();
    let _hash2 = accumulator.append(&[HashValue::sha3_256_of(b"b")]).unwrap();
    let _hash3 = accumulator.append(&[HashValue::sha3_256_of(b"c")]).unwrap();
    let accumulator_info = accumulator.get_info();
    let _hash4 = accumulator.append(&[HashValue::sha3_256_of(b"d")]).unwrap();

    assert_eq!(
        HashValue::sha3_256_of(b"b"),
        accumulator.get_leaf(1).unwrap().unwrap()
    );
    accumulator.flush().unwrap();
    accumulator = accumulator.fork(Some(accumulator_info));
    let _hash5 = accumulator.append(&[HashValue::sha3_256_of(b"e")]).unwrap();

    assert_eq!(
        HashValue::sha3_256_of(b"b"),
        accumulator.get_leaf(1).unwrap().unwrap()
    );
    assert_eq!(
        HashValue::sha3_256_of(b"c"),
        accumulator.get_leaf(2).unwrap().unwrap()
    );
    assert_eq!(
        HashValue::sha3_256_of(b"e"),
        accumulator.get_leaf(3).unwrap().unwrap()
    );
    assert_ne!(
        HashValue::sha3_256_of(b"d"),
        accumulator.get_leaf(3).unwrap().unwrap()
    );
}
