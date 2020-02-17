use std::cmp::Ordering;
use std::collections::{BTreeSet, HashSet};
use std::marker::PhantomData;
use std::time::Duration;
use types::{block::BlockNumber, peer_info::PeerInfo};

#[derive(Eq, PartialEq, Debug)]
struct TTLEntry<E>
where
    E: Ord,
{
    phantom: PhantomData<E>,
    expiration_time: Duration,
    height: BlockNumber,
    peers: HashSet<PeerInfo>,
}

impl<E> TTLEntry<E>
where
    E: Ord,
{
    fn expiration_time(&self) -> Duration {
        self.expiration_time
    }
}

impl<E> PartialOrd for TTLEntry<E>
where
    E: Ord,
{
    fn partial_cmp(&self, other: &TTLEntry<E>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<E> Ord for TTLEntry<E>
where
    E: Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        match self.height.cmp(&other.height) {
            Ordering::Equal => {
                return self.phantom.cmp(&other.phantom);
            }
            ordering => return ordering,
        }
    }
}

pub struct TTLPool<E>
where
    E: Ord,
{
    data: BTreeSet<TTLEntry<E>>,
}

impl<E> TTLPool<E>
where
    E: Ord,
{
    pub(crate) fn new() -> Self {
        Self {
            data: BTreeSet::new(),
        }
    }

    /// add transaction to index
    pub(crate) fn insert(&mut self, entry: &E) {
        //todo
    }

    /// remove transaction from index
    pub(crate) fn remove(&mut self, entry: &E) {
        //todo
    }

    pub(crate) fn gc(&mut self, now: Duration) -> Vec<E> {
        //todo
        unimplemented!()
    }

    pub(crate) fn size(&self) -> usize {
        self.data.len()
    }
}
