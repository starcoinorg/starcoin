// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::MVHashMap;
use proptest::{collection::vec, prelude::*, sample::Index, strategy::Strategy};
use std::{
    collections::{BTreeMap, HashMap},
    fmt::Debug,
    hash::Hash,
    sync::atomic::{AtomicUsize, Ordering},
};

const DEFAULT_TIMEOUT: u64 = 30;

#[derive(Debug, Clone)]
enum Operator<V: Debug + Clone> {
    Insert(V),
    Remove,
    Read,
}

#[derive(Debug, Clone, PartialEq)]
enum ExpectedOutput<V: Debug + Clone + PartialEq> {
    NotInMap,
    Deleted,
    Value(V),
}

struct Baseline<K, V>(HashMap<K, BTreeMap<usize, Option<V>>>);

impl<K, V> Baseline<K, V>
where
    K: Hash + Eq + Clone,
    V: Clone + Debug + PartialEq,
{
    pub fn new(txns: &[(K, Operator<V>)]) -> Self {
        let mut baseline: HashMap<K, BTreeMap<usize, Option<V>>> = HashMap::new();
        for (idx, (k, op)) in txns.iter().enumerate() {
            let value_to_update = match op {
                Operator::Insert(v) => Some(v.clone()),
                Operator::Remove => None,
                Operator::Read => continue,
            };

            baseline
                .entry(k.clone())
                .or_insert_with(BTreeMap::new)
                .insert(idx, value_to_update);
        }
        Self(baseline)
    }

    pub fn get(&self, key: &K, version: usize) -> ExpectedOutput<V> {
        match self
            .0
            .get(key)
            .and_then(|tree| tree.range(..version).last())
        {
            None => ExpectedOutput::NotInMap,
            Some((_, Some(v))) => ExpectedOutput::Value(v.clone()),
            Some((_, None)) => ExpectedOutput::Deleted,
        }
    }
}

fn operator_strategy<V: Arbitrary + Clone>() -> impl Strategy<Value = Operator<V>> {
    prop_oneof![
        2 => any::<V>().prop_map(Operator::Insert),
        1 => Just(Operator::Remove),
        4 => Just(Operator::Read),
    ]
}

fn run_and_assert<K, V>(
    universe: Vec<K>,
    transaction_gens: Vec<(Index, Operator<V>)>,
) -> Result<(), TestCaseError>
where
    K: PartialOrd + Send + Clone + Hash + Eq + Sync,
    V: Send + Debug + Clone + PartialEq + Sync,
{
    let transactions: Vec<(K, Operator<V>)> = transaction_gens
        .into_iter()
        .map(|(idx, op)| (idx.get(&universe).clone(), op))
        .collect::<Vec<_>>();

    let baseline = Baseline::new(transactions.as_slice());
    let map = MVHashMap::<K, Option<V>>::new();

    // make ESTIMATE placeholders for all versions to be written.
    // allows to test that correct values appear at the end of concurrent execution.
    let versions_to_write = transactions
        .iter()
        .enumerate()
        .filter_map(|(idx, (key, op))| match op {
            Operator::Read => None,
            Operator::Insert(_) | Operator::Remove => Some((key.clone(), idx)),
        })
        .collect::<Vec<_>>();
    for (key, idx) in versions_to_write {
        map.write(&key, (idx, 0), None);
        map.mark_estimate(&key, idx);
    }

    let curent_idx = AtomicUsize::new(0);

    // Spawn a few threads in parallel to commit each operator.
    rayon::scope(|s| {
        for _ in 0..universe.len() {
            s.spawn(|_| loop {
                // Each thread will eagerly fetch an Operator to execute.
                let idx = curent_idx.fetch_add(1, Ordering::Relaxed);
                if idx >= transactions.len() {
                    // Abort when all transactions are processed.
                    break;
                }
                let key = &transactions[idx].0;
                match &transactions[idx].1 {
                    Operator::Read => {
                        let baseline = baseline.get(key, idx);
                        let mut retry_attempts = 0;
                        loop {
                            match map.read(key, idx) {
                                Ok((_, v)) => {
                                    match &*v {
                                        Some(w) => {
                                            assert_eq!(
                                                baseline,
                                                ExpectedOutput::Value(w.clone()),
                                                "{:?}",
                                                idx
                                            );
                                        }
                                        None => {
                                            assert_eq!(
                                                baseline,
                                                ExpectedOutput::Deleted,
                                                "{:?}",
                                                idx
                                            );
                                        }
                                    }
                                    break;
                                }
                                Err(None) => {
                                    assert_eq!(baseline, ExpectedOutput::NotInMap, "{:?}", idx);
                                    break;
                                }
                                Err(Some(_i)) => (),
                            }
                            retry_attempts += 1;
                            if retry_attempts > DEFAULT_TIMEOUT {
                                panic!("Failed to get value for {:?}", idx);
                            }
                            std::thread::sleep(std::time::Duration::from_millis(100));
                        }
                    }
                    Operator::Remove => {
                        map.write(key, (idx, 1), None);
                    }
                    Operator::Insert(v) => {
                        map.write(key, (idx, 1), Some(v.clone()));
                    }
                }
            })
        }
    });
    Ok(())
}

proptest! {
    #[test]
    fn single_key_proptest(
        universe in vec(any::<[u8; 32]>(), 1),
        transactions in vec((any::<Index>(), operator_strategy::<[u8; 32]>()), 100),
    ) {
        run_and_assert(universe, transactions)?;
    }

    #[test]
    fn single_key_large_transactions(
        universe in vec(any::<[u8; 32]>(), 1),
        transactions in vec((any::<Index>(), operator_strategy::<[u8; 32]>()), 2000),
    ) {
        run_and_assert(universe, transactions)?;
    }

    #[test]
    fn multi_key_proptest(
        universe in vec(any::<[u8; 32]>(), 10),
        transactions in vec((any::<Index>(), operator_strategy::<[u8; 32]>()), 100),
    ) {
        run_and_assert(universe, transactions)?;
    }
}
