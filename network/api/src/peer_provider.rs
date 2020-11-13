// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::PeerId;
use crate::PeerInfo;
use anyhow::{format_err, Result};
use futures::future::BoxFuture;
use futures::{FutureExt, TryFutureExt};
use itertools::Itertools;
use rand::prelude::IteratorRandom;
use rand::prelude::SliceRandom;
use starcoin_types::block::BlockNumber;

pub trait PeerProvider: Send + Sync {
    fn identify(&self) -> PeerId;

    fn best_peer(&self) -> BoxFuture<Result<Option<PeerInfo>>> {
        self.peer_selector()
            .and_then(|selector| async move { Ok(selector.bests().random().cloned()) })
            .boxed()
    }

    /// Get all peers, the peer's order is unsorted.
    fn peer_set(&self) -> BoxFuture<Result<Vec<PeerInfo>>>;

    fn get_peer(&self, peer_id: PeerId) -> BoxFuture<Result<Option<PeerInfo>>>;

    fn get_self_peer(&self) -> BoxFuture<Result<PeerInfo>> {
        let peer_id = self.identify();
        self.get_peer(peer_id)
            .and_then(|result| async move {
                result.ok_or_else(|| format_err!("Can not find peer by self id"))
            })
            .boxed()
    }

    fn peer_selector(&self) -> BoxFuture<Result<PeerSelector>> {
        self.peer_set()
            .and_then(|peers| async move { Ok(PeerSelector::new(peers)) })
            .boxed()
    }
}

pub struct Selector<'a> {
    peers: Vec<&'a PeerInfo>,
}

impl<'a> Selector<'a> {
    pub fn new(peers: Vec<&'a PeerInfo>) -> Self {
        Self { peers }
    }

    pub fn empty() -> Self {
        Self::new(vec![])
    }

    pub fn first(&self) -> Option<&PeerInfo> {
        self.peers.get(0).copied()
    }

    pub fn random_peer_id(&self) -> Option<PeerId> {
        self.random().map(|info| info.peer_id.clone())
    }

    pub fn random(&self) -> Option<&PeerInfo> {
        self.peers.choose(&mut rand::thread_rng()).copied()
    }

    pub fn is_empty(&self) -> bool {
        self.peers.is_empty()
    }

    pub fn len(&self) -> usize {
        self.peers.len()
    }

    pub fn filter<P>(self, predicate: P) -> Selector<'a>
    where
        P: Fn(&PeerInfo) -> bool + Send + 'static,
    {
        if self.is_empty() {
            return self;
        }
        Selector::new(
            self.peers
                .into_iter()
                .filter(|peer| predicate(peer))
                .collect(),
        )
    }

    pub fn filter_by_block_number(self, block_number: BlockNumber) -> Selector<'a> {
        self.filter(move |peer| peer.latest_header.number >= block_number)
    }

    pub fn cloned(self) -> Vec<PeerInfo> {
        self.peers.into_iter().cloned().collect()
    }

    pub fn into_selector(self) -> PeerSelector {
        PeerSelector::new(self.cloned())
    }
}

impl<'a> From<Vec<&'a PeerInfo>> for Selector<'a> {
    fn from(peers: Vec<&'a PeerInfo>) -> Self {
        Self::new(peers)
    }
}

#[derive(Clone)]
pub struct PeerSelector {
    peers: Vec<PeerInfo>,
}

impl PeerSelector {
    pub fn new(peers: Vec<PeerInfo>) -> Self {
        Self { peers }
    }

    /// Get top N peers sorted by total_difficulty
    pub fn top(self, n: usize) -> Self {
        if self.is_empty() {
            return self;
        }
        let mut peers = self.peers;
        Self::sort(&mut peers);
        Self::new(peers.into_iter().take(n).collect())
    }

    /// Filter by the max total_difficulty
    pub fn bests(&self) -> Selector {
        if self.is_empty() {
            return Selector::empty();
        }
        let peers: Vec<&PeerInfo> = vec![];
        let best_peers = self
            .peers
            .iter()
            .sorted_by_key(|info| info.total_difficulty)
            .rev()
            .fold(peers, |mut peers, peer| {
                if peers.is_empty() || peer.total_difficulty >= peers[0].total_difficulty {
                    peers.push(peer);
                };
                peers
            });
        Selector::new(best_peers)
    }

    pub fn selector(&self) -> Selector {
        Selector::new(self.peers.as_slice().iter().collect())
    }

    pub fn random_peer_id(&self) -> Option<PeerId> {
        self.peers
            .iter()
            .choose(&mut rand::thread_rng())
            .map(|info| info.peer_id.clone())
    }

    pub fn random(&self) -> Option<&PeerInfo> {
        self.peers.iter().choose(&mut rand::thread_rng())
    }

    pub fn peers(&self) -> &[PeerInfo] {
        self.peers.as_slice()
    }

    pub fn first(&self) -> Option<&PeerInfo> {
        self.peers.get(0)
    }

    pub fn is_empty(&self) -> bool {
        self.peers.is_empty()
    }

    pub fn len(&self) -> usize {
        self.peers.len()
    }

    fn sort(peers: &mut Vec<PeerInfo>) {
        peers.sort_by_key(|p| p.total_difficulty);
        peers.reverse();
    }
}

impl IntoIterator for PeerSelector {
    type Item = PeerInfo;
    type IntoIter = std::vec::IntoIter<PeerInfo>;

    fn into_iter(self) -> Self::IntoIter {
        self.peers.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use crate::peer_provider::PeerSelector;
    use starcoin_types::block::BlockHeader;
    use starcoin_types::peer_info::{PeerId, PeerInfo};

    #[test]
    fn test_peer_selector() {
        let peers = vec![
            PeerInfo::new_with_proto(PeerId::random(), 100.into(), BlockHeader::random(), vec![]),
            PeerInfo::new_with_proto(PeerId::random(), 99.into(), BlockHeader::random(), vec![]),
            PeerInfo::new_with_proto(PeerId::random(), 100.into(), BlockHeader::random(), vec![]),
            PeerInfo::new_with_proto(PeerId::random(), 1.into(), BlockHeader::random(), vec![]),
        ];

        let peer_selector = PeerSelector::new(peers);
        let beat_selector = peer_selector.bests();
        assert_eq!(2, beat_selector.len());

        let top_selector = peer_selector.top(3);
        assert_eq!(3, top_selector.len());
    }
}
