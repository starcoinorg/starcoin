use std::str::FromStr;
use std::sync::Arc;

use starcoin_crypto::HashValue;
use starcoin_storage::db_storage::DBStorage;
use starcoin_storage::storage::StorageInstance;
use starcoin_storage::BlockStore;
use starcoin_storage::Storage;
use starcoin_storage::StorageVersion;

fn main() {
    let db_storage = DBStorage::open_with_cfs(
        "/home/fikgol/.starcoin/main/starcoindb/db/starcoindb",
        StorageVersion::V3.get_column_family_names().to_vec(),
        true,
        Default::default(),
        None,
    )
    .unwrap();
    let storage = Storage::new(StorageInstance::new_db_instance(db_storage)).unwrap();
    let storage = Arc::new(storage);
    let mut stop = false;
    let stop_hash: HashValue =
        HashValue::from_str("0x80848150abee7e9a3bfe9542a019eb0b8b01f124b63b011f9c338fdb935c417d")
            .unwrap();
    let mut search_hash: HashValue =
        HashValue::from_str("0x7cd4b95c72fc9db05053187d061e5c11ee0e6174271526658661f5bb51eb41b1")
            .unwrap();
    while stop {
        let block = storage.get_block_by_hash(search_hash).unwrap().unwrap();
        for tx in block.transactions() {
            let raw_txn = tx.raw_txn();
            let address = raw_txn.sender().to_hex();
            println!("{}", address);
        }
        search_hash = block.header().parent_hash();
        if stop_hash == search_hash {
            stop = true;
        }
    }
}
