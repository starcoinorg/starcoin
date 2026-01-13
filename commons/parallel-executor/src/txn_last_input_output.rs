// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    delayed_field::DelayedFieldReads,
    errors::Error,
    scheduler::{Incarnation, TxnIndex, Version},
    task::{ExecutionStatus, Transaction, TransactionOutput},
};
use arc_swap::ArcSwapOption;
use crossbeam::utils::CachePadded;
use std::{collections::HashSet, sync::Arc};

type TxnInput<K> = Vec<ReadDescriptor<K>>;
type TxnOutput<T, E> = ExecutionStatus<T, Error<E>>;
type TxnDelayedInput = DelayedFieldReads;

// If an entry was read from the multi-version data-structure, then kind is
// MVHashMap(txn_idx, incarnation), with transaction index and incarnation number
// of the execution associated with the write of the entry. Otherwise, if the read
// occurred from storage, and kind is set to Storage.
#[derive(Clone, PartialEq)]
enum ReadKind {
    MVHashMap(TxnIndex, Incarnation),
    Storage,
}

#[derive(Clone)]
pub struct ReadDescriptor<K> {
    access_path: K,

    kind: ReadKind,
}

impl<K> ReadDescriptor<K> {
    pub fn from(access_path: K, txn_idx: TxnIndex, incarnation: Incarnation) -> Self {
        Self {
            access_path,
            kind: ReadKind::MVHashMap(txn_idx, incarnation),
        }
    }

    pub fn from_storage(access_path: K) -> Self {
        Self {
            access_path,
            kind: ReadKind::Storage,
        }
    }

    pub fn path(&self) -> &K {
        &self.access_path
    }

    // Does the read descriptor describe a read from MVHashMap w. a specified version.
    pub fn validate_version(&self, version: Version) -> bool {
        let (txn_idx, incarnation) = version;
        self.kind == ReadKind::MVHashMap(txn_idx, incarnation)
    }

    // Does the read descriptor describe a read from storage.
    pub fn validate_storage(&self) -> bool {
        self.kind == ReadKind::Storage
    }
}

pub struct TxnLastInputOutput<K, T, E> {
    inputs: Vec<CachePadded<ArcSwapOption<TxnInput<K>>>>, // txn_idx -> input.
    delayed_inputs: Vec<CachePadded<ArcSwapOption<TxnDelayedInput>>>,

    outputs: Vec<CachePadded<ArcSwapOption<TxnOutput<T, E>>>>, // txn_idx -> output.
}

impl<K, T: TransactionOutput, E: Send + Clone> TxnLastInputOutput<K, T, E> {
    pub fn new(num_txns: usize) -> Self {
        Self {
            inputs: (0..num_txns)
                .map(|_| CachePadded::new(ArcSwapOption::empty()))
                .collect(),
            delayed_inputs: (0..num_txns)
                .map(|_| CachePadded::new(ArcSwapOption::empty()))
                .collect(),
            outputs: (0..num_txns)
                .map(|_| CachePadded::new(ArcSwapOption::empty()))
                .collect(),
        }
    }

    pub fn record(
        &self,
        txn_idx: TxnIndex,
        input: Vec<ReadDescriptor<K>>,
        delayed_input: DelayedFieldReads,
        output: ExecutionStatus<T, Error<E>>,
    ) {
        self.inputs[txn_idx].store(Some(Arc::new(input)));
        self.delayed_inputs[txn_idx].store(Some(Arc::new(delayed_input)));
        self.outputs[txn_idx].store(Some(Arc::new(output)));
    }

    pub fn read_set(&self, txn_idx: TxnIndex) -> Option<Arc<Vec<ReadDescriptor<K>>>> {
        self.inputs[txn_idx].load_full()
    }

    pub fn delayed_read_set(&self, txn_idx: TxnIndex) -> Option<Arc<DelayedFieldReads>> {
        self.delayed_inputs[txn_idx].load_full()
    }

    // Extracts a set of paths written during execution from transaction output.
    pub fn write_set(
        &self,
        txn_idx: TxnIndex,
    ) -> HashSet<<<T as TransactionOutput>::T as Transaction>::Key> {
        match &self.outputs[txn_idx].load_full() {
            None => HashSet::new(),
            Some(txn_output) => match txn_output.as_ref() {
                ExecutionStatus::Success(t) | ExecutionStatus::SkipRest(t) => {
                    t.get_writes().into_iter().map(|(k, _)| k).collect()
                }
                ExecutionStatus::Abort(_) => HashSet::new(),
            },
        }
    }

    // Must be executed after parallel execution is done, grabs outputs. Will panic if
    // other outstanding references to the recorded outputs exist.
    pub fn take_output(&self, txn_idx: TxnIndex) -> ExecutionStatus<T, Error<E>> {
        let owning_ptr = self.outputs[txn_idx]
            .swap(None)
            .unwrap_or_else(|| {
                panic!("Output must be recorded after execution (txn_idx={})", txn_idx)
            });

        match Arc::try_unwrap(owning_ptr) {
            Ok(output) => output,
            _ => {
                unreachable!("Output should be uniquely owned after execution");
            }
        }
    }

    // Get gas_used from the transaction output without consuming it
    pub fn gas_used(&self, txn_idx: TxnIndex) -> u64 {
        match &self.outputs[txn_idx].load_full() {
            None => 0,
            Some(txn_output) => match txn_output.as_ref() {
                ExecutionStatus::Success(t) | ExecutionStatus::SkipRest(t) => t.gas_used(),
                ExecutionStatus::Abort(_) => 0,
            },
        }
    }
}
