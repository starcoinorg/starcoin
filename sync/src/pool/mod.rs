use std::cmp::Ordering;
use std::collections::{BTreeSet, HashSet};
use std::sync::RwLock;
use std::time::Duration;
use types::{block::BlockNumber, peer_info::PeerId};

#[derive(Eq, PartialEq, Clone, Debug)]
struct TTLEntry<E>
where
    E: Ord + Clone,
{
    data: E,
    expiration_time: Duration,
    block_number: BlockNumber,
    peers: HashSet<PeerId>,
}

impl<E> TTLEntry<E>
where
    E: Ord + Clone,
{
    fn _expiration_time(&self) -> Duration {
        self.expiration_time
    }

    fn new(peer: PeerId, block_number: BlockNumber, entry: E) -> Self {
        let mut peers = HashSet::new();
        peers.insert(peer);
        TTLEntry {
            data: entry,
            expiration_time: Duration::from_secs(60 * 60),
            block_number,
            peers,
        }
    }
}

impl<E> PartialOrd for TTLEntry<E>
where
    E: Ord + Clone,
{
    fn partial_cmp(&self, other: &TTLEntry<E>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<E> Ord for TTLEntry<E>
where
    E: Ord + Clone,
{
    fn cmp(&self, other: &Self) -> Ordering {
        match self.block_number.cmp(&other.block_number) {
            Ordering::Equal => self.data.cmp(&other.data),
            ordering => ordering,
        }
    }
}

/// thread safe
pub struct TTLPool<E>
where
    E: Ord + Clone,
{
    data: RwLock<BTreeSet<TTLEntry<E>>>,
}

impl<E> TTLPool<E>
where
    E: Ord + Clone,
{
    pub(crate) fn new() -> Self {
        Self {
            data: RwLock::new(BTreeSet::new()),
        }
    }

    /// add entry to pool
    pub(crate) fn insert(&self, peer: PeerId, number: BlockNumber, entry: E) {
        let mut ttl_entry = TTLEntry::new(peer.clone(), number, entry);
        let mut lock = self.data.write().unwrap();
        if lock.contains(&ttl_entry) {
            ttl_entry = lock.take(&ttl_entry).expect("entry not exist.")
        };

        ttl_entry.peers.insert(peer);
        lock.insert(ttl_entry);
    }

    /// take entry from pool
    pub(crate) fn take(&self, size: usize) -> Vec<E> {
        let mut lock = self.data.write().unwrap();
        let mut set_iter = lock.iter();
        let mut entries = Vec::new();
        loop {
            if entries.len() >= size {
                break;
            }

            let entry = set_iter.next();

            if entry.is_none() {
                break;
            }

            let ttl_entry = entry.expect("entry is none.").clone();
            entries.push(ttl_entry);
        }

        drop(set_iter);

        if !entries.is_empty() {
            entries.iter().for_each(|e| {
                lock.remove(e);
            });
        }

        entries.iter().map(|e| e.data.clone()).collect()
    }

    pub(crate) fn _gc(&self, _now: Duration) -> Vec<E> {
        //todo
        unimplemented!()
    }

    pub(crate) fn _size(&self) -> usize {
        self.data.read().unwrap().len()
    }
}
