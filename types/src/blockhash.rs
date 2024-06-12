use starcoin_crypto::hash::HashValue;
use std::collections::{HashMap, HashSet};

pub const BLOCK_VERSION: u16 = 1;

pub const HASH_LENGTH: usize = HashValue::LENGTH;

use starcoin_uint::U256;
use std::sync::Arc;

pub type BlockHashes = Arc<Vec<HashValue>>;

/// `blockhash::NONE` is a hash which is used in rare cases as the `None` block hash
pub const NONE: [u8; HASH_LENGTH] = [0u8; HASH_LENGTH];

/// `blockhash::VIRTUAL` is a special hash representing the `virtual` block.
pub const VIRTUAL: [u8; HASH_LENGTH] = [0xff; HASH_LENGTH];

/// `blockhash::ORIGIN` is a special hash representing a `virtual genesis` block.
/// It serves as a special local block which all locally-known
/// blocks are in its future.
pub const ORIGIN: [u8; HASH_LENGTH] = [0xfe; HASH_LENGTH];

pub trait BlockHashExtensions {
    fn is_none(&self) -> bool;
    fn is_virtual(&self) -> bool;
    fn is_origin(&self) -> bool;
}

impl BlockHashExtensions for HashValue {
    fn is_none(&self) -> bool {
        self.eq(&HashValue::new(NONE))
    }

    fn is_virtual(&self) -> bool {
        self.eq(&HashValue::new(VIRTUAL))
    }

    fn is_origin(&self) -> bool {
        self.eq(&HashValue::new(ORIGIN))
    }
}

/// Generates a unique block hash for each call to this function.
/// To be used for test purposes only.
pub fn new_unique() -> HashValue {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    let c = COUNTER.fetch_add(1, Ordering::Relaxed);
    HashValue::from_u64(c)
}

pub type BlueWorkType = U256;

/// The type used to represent the GHOSTDAG K parameter
pub type KType = u16;

/// Map from Block hash to K type
pub type HashKTypeMap = std::sync::Arc<BlockHashMap<KType>>;

pub type BlockHashMap<V> = HashMap<HashValue, V>;

/// Same as `BlockHashMap` but a `HashSet`.
pub type BlockHashSet = HashSet<HashValue>;

pub struct ChainPath {
    pub added: Vec<HashValue>,
    pub removed: Vec<HashValue>,
}

pub type BlockLevel = u8;

pub trait BlockHashIteratorExtensions: Iterator<Item = HashValue> {
    /// Copy of itertools::unique, adapted for block hashes (uses `BlockHashSet` under the hood)
    ///
    /// Returns an iterator adaptor that filters out hashes that have
    /// already been produced once during the iteration.
    ///
    /// Clones of visited elements are stored in a hash set in the
    /// iterator.
    ///
    /// The iterator is stable, returning the non-duplicate items in the order
    /// in which they occur in the adapted iterator. In a set of duplicate
    /// items, the first item encountered is the item retained.
    ///
    /// NOTE: currently usages are expected to contain no duplicates, hence we alloc the expected capacity
    fn block_unique(self) -> BlockUnique<Self>
    where
        Self: Sized,
    {
        let (lower, _) = self.size_hint();
        BlockUnique {
            iter: self,
            seen: BlockHashSet::with_capacity(lower),
        }
    }
}

impl<T: ?Sized> BlockHashIteratorExtensions for T where T: Iterator<Item = HashValue> {}

#[derive(Clone)]
pub struct BlockUnique<I: Iterator<Item = HashValue>> {
    iter: I,
    seen: BlockHashSet,
}

impl<I> Iterator for BlockUnique<I>
where
    I: Iterator<Item = HashValue>,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.by_ref().find(|&hash| self.seen.insert(hash))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (low, hi) = self.iter.size_hint();
        ((low > 0 && self.seen.is_empty()) as usize, hi)
    }
}
