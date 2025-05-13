use starcoin_crypto::hash::HashValue;
use std::{
    collections::{HashMap, HashSet},
    hash::{BuildHasher, Hasher},
};

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
        self.eq(&Self::new(NONE))
    }

    fn is_virtual(&self) -> bool {
        self.eq(&Self::new(VIRTUAL))
    }

    fn is_origin(&self) -> bool {
        self.eq(&Self::new(ORIGIN))
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

/// `hashes::Hash` writes 4 u64s so we just use the last one as the hash here
#[derive(Default, Clone, Copy)]
pub struct BlockHasher(u64);

impl BlockHasher {
    #[inline(always)]
    pub const fn new() -> Self {
        Self(0)
    }
}

impl Hasher for BlockHasher {
    #[inline(always)]
    fn finish(&self) -> u64 {
        self.0
    }
    #[inline(always)]
    fn write_u64(&mut self, v: u64) {
        self.0 = v;
    }
    #[cold]
    fn write(&mut self, _: &[u8]) {
        unimplemented!("use write_u64")
    }
}

impl BuildHasher for BlockHasher {
    type Hasher = Self;

    #[inline(always)]
    fn build_hasher(&self) -> Self::Hasher {
        Self(0)
    }
}

pub type BlockLevel = u8;
