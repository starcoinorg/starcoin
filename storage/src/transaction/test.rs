// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

// use crate::LibraDB;
use crate::cache_storage::CacheStorage;
use crate::db_storage::DBStorage;
use crate::storage::StorageInstance;
use crate::{Storage, TransactionStore};
use proptest::{collection::vec, prelude::*};
use starcoin_types::{
    proptest_types::{AccountInfoUniverse, Index, SignatureCheckedTransactionGen},
    transaction::Transaction,
};
use std::sync::Arc;

fn init_store(
    mut universe: AccountInfoUniverse,
    gens: Vec<(Index, SignatureCheckedTransactionGen)>,
    store: &Storage,
) -> Vec<Transaction> {
    let txns = gens
        .into_iter()
        .map(|(index, gen)| {
            Transaction::UserTransaction(gen.materialize(index, &mut universe).into_inner())
        })
        .collect::<Vec<_>>();
    store
        .transaction_storage
        .save_transaction_batch(txns.clone())
        .unwrap();
    txns
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn test_put_get(
        universe in any_with::<AccountInfoUniverse>(3),
        gens in vec(
            (any::<Index>(), any::<SignatureCheckedTransactionGen>()),
            1..10
        ),
    ) {
        let cache_storage = Arc::new(CacheStorage::new());
        let tmpdir = starcoin_config::temp_path();
        let db_storage = Arc::new(DBStorage::new(tmpdir.path()));
        let storage = Storage::new(StorageInstance::new_cache_and_db_instance(
            cache_storage,
            db_storage,
        ))
        .unwrap();
        let txns = init_store(universe, gens, &storage);

        for (_ver, txn) in txns.iter().enumerate() {
            prop_assert_eq!(storage
                            .transaction_storage
                            .get_transaction(txn.id()).unwrap().unwrap(), txn.clone());
        }
    }
}
