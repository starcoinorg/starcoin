//!
//! Test utils for reachability
//!
use super::{inquirer::*, tree::*};
use crate::consensusdb::{
    prelude::StoreError,
    schemadb::{ReachabilityStore, ReachabilityStoreReader},
};
use crate::dag::types::{interval::Interval, perf};
use starcoin_crypto::HashValue as Hash;
use starcoin_types::blockhash::{BlockHashExtensions, BlockHashMap, BlockHashSet};
use std::collections::VecDeque;
use thiserror::Error;

/// A struct with fluent API to streamline reachability store building
pub struct StoreBuilder<'a, T: ReachabilityStore + ?Sized> {
    store: &'a mut T,
}

impl<'a, T: ReachabilityStore + ?Sized> StoreBuilder<'a, T> {
    pub fn new(store: &'a mut T) -> Self {
        Self { store }
    }

    pub fn add_block(&mut self, hash: Hash, parent: Hash) -> &mut Self {
        let parent_height = if !parent.is_none() {
            self.store.append_child(parent, hash).unwrap()
        } else {
            0
        };
        self.store
            .insert(hash, parent, Interval::empty(), parent_height + 1)
            .unwrap();
        self
    }
}

/// A struct with fluent API to streamline tree building
pub struct TreeBuilder<'a, T: ReachabilityStore + ?Sized> {
    store: &'a mut T,
    reindex_depth: u64,
    reindex_slack: u64,
}

impl<'a, T: ReachabilityStore + ?Sized> TreeBuilder<'a, T> {
    pub fn new(store: &'a mut T) -> Self {
        Self {
            store,
            reindex_depth: perf::DEFAULT_REINDEX_DEPTH,
            reindex_slack: perf::DEFAULT_REINDEX_SLACK,
        }
    }

    pub fn new_with_params(store: &'a mut T, reindex_depth: u64, reindex_slack: u64) -> Self {
        Self {
            store,
            reindex_depth,
            reindex_slack,
        }
    }

    pub fn init(&mut self) -> &mut Self {
        init(self.store).unwrap();
        self
    }

    pub fn init_with_params(&mut self, origin: Hash, capacity: Interval) -> &mut Self {
        init_with_params(self.store, origin, capacity).unwrap();
        self
    }

    pub fn add_block(&mut self, hash: Hash, parent: Hash) -> &mut Self {
        add_tree_block(
            self.store,
            hash,
            parent,
            self.reindex_depth,
            self.reindex_slack,
        )
        .unwrap();
        try_advancing_reindex_root(self.store, hash, self.reindex_depth, self.reindex_slack)
            .unwrap();
        self
    }

    pub fn store(&self) -> &&'a mut T {
        &self.store
    }
}

#[derive(Clone)]
pub struct DagBlock {
    pub hash: Hash,
    pub parents: Vec<Hash>,
}

impl DagBlock {
    pub fn new(hash: Hash, parents: Vec<Hash>) -> Self {
        Self { hash, parents }
    }
}

/// A struct with fluent API to streamline DAG building
pub struct DagBuilder<'a, T: ReachabilityStore + ?Sized> {
    store: &'a mut T,
    map: BlockHashMap<DagBlock>,
}

impl<'a, T: ReachabilityStore + ?Sized> DagBuilder<'a, T> {
    pub fn new(store: &'a mut T) -> Self {
        Self {
            store,
            map: BlockHashMap::new(),
        }
    }

    pub fn init(&mut self) -> &mut Self {
        init(self.store).unwrap();
        self
    }

    pub fn add_block(&mut self, block: DagBlock) -> &mut Self {
        // Select by height (longest chain) just for the sake of internal isolated tests
        let selected_parent = block
            .parents
            .iter()
            .cloned()
            .max_by_key(|p| self.store.get_height(*p).unwrap())
            .unwrap();
        let mergeset = self.mergeset(&block, selected_parent);
        add_block(
            self.store,
            block.hash,
            selected_parent,
            &mut mergeset.iter().cloned(),
        )
        .unwrap();
        hint_virtual_selected_parent(self.store, block.hash).unwrap();
        self.map.insert(block.hash, block);
        self
    }

    fn mergeset(&self, block: &DagBlock, selected_parent: Hash) -> Vec<Hash> {
        let mut queue: VecDeque<Hash> = block
            .parents
            .iter()
            .copied()
            .filter(|p| *p != selected_parent)
            .collect();
        let mut mergeset: BlockHashSet = queue.iter().copied().collect();
        let mut past = BlockHashSet::new();

        while let Some(current) = queue.pop_front() {
            for parent in self.map[&current].parents.iter() {
                if mergeset.contains(parent) || past.contains(parent) {
                    continue;
                }

                if is_dag_ancestor_of(self.store, *parent, selected_parent).unwrap() {
                    past.insert(*parent);
                    continue;
                }

                mergeset.insert(*parent);
                queue.push_back(*parent);
            }
        }
        mergeset.into_iter().collect()
    }

    pub fn store(&self) -> &&'a mut T {
        &self.store
    }
}

#[derive(Error, Debug)]
pub enum TestError {
    #[error("data store error")]
    StoreError(#[from] StoreError),

    #[error("empty interval")]
    EmptyInterval(Hash, Interval),

    #[error("sibling intervals are expected to be consecutive")]
    NonConsecutiveSiblingIntervals(Interval, Interval),

    #[error("child interval out of parent bounds")]
    IntervalOutOfParentBounds {
        parent: Hash,
        child: Hash,
        parent_interval: Interval,
        child_interval: Interval,
    },
}

pub trait StoreValidationExtensions {
    /// Checks if `block` is in the past of `other` (creates hashes from the u64 numbers)
    fn in_past_of(&self, block: u64, other: u64) -> bool;

    /// Checks if `block` and `other` are in the anticone of each other
    /// (creates hashes from the u64 numbers)
    fn are_anticone(&self, block: u64, other: u64) -> bool;

    /// Validates that all tree intervals match the expected interval relations
    fn validate_intervals(&self, root: Hash) -> std::result::Result<(), TestError>;
}

impl<T: ReachabilityStoreReader + ?Sized> StoreValidationExtensions for T {
    fn in_past_of(&self, block: u64, other: u64) -> bool {
        if block == other {
            return false;
        }
        let res = is_dag_ancestor_of(self, block.into(), other.into()).unwrap();
        if res {
            // Assert that the `future` relation is indeed asymmetric
            assert!(!is_dag_ancestor_of(self, other.into(), block.into()).unwrap())
        }
        res
    }

    fn are_anticone(&self, block: u64, other: u64) -> bool {
        !is_dag_ancestor_of(self, block.into(), other.into()).unwrap()
            && !is_dag_ancestor_of(self, other.into(), block.into()).unwrap()
    }

    fn validate_intervals(&self, root: Hash) -> std::result::Result<(), TestError> {
        let mut queue = VecDeque::<Hash>::from([root]);
        while let Some(parent) = queue.pop_front() {
            let children = self.get_children(parent)?;
            queue.extend(children.iter());

            let parent_interval = self.get_interval(parent)?;
            if parent_interval.is_empty() {
                return Err(TestError::EmptyInterval(parent, parent_interval));
            }

            // Verify parent-child strict relation
            for child in children.iter().cloned() {
                let child_interval = self.get_interval(child)?;
                if !parent_interval.strictly_contains(child_interval) {
                    return Err(TestError::IntervalOutOfParentBounds {
                        parent,
                        child,
                        parent_interval,
                        child_interval,
                    });
                }
            }

            // Iterate over consecutive siblings
            for siblings in children.windows(2) {
                let sibling_interval = self.get_interval(siblings[0])?;
                let current_interval = self.get_interval(siblings[1])?;
                if sibling_interval.end + 1 != current_interval.start {
                    return Err(TestError::NonConsecutiveSiblingIntervals(
                        sibling_interval,
                        current_interval,
                    ));
                }
            }
        }
        Ok(())
    }
}
