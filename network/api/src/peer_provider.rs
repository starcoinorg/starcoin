// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::peer_score::ScoreCounter;
use crate::PeerId;
use crate::PeerInfo;
use anyhow::Result;
use futures::future::BoxFuture;
use futures::{FutureExt, TryFutureExt};
use itertools::Itertools;
use rand::prelude::IteratorRandom;
use rand::prelude::SliceRandom;
use starcoin_types::block::{BlockHeader, BlockNumber};

pub trait PeerProvider: Send + Sync {
    fn best_peer(&self) -> BoxFuture<Result<Option<PeerInfo>>> {
        self.peer_selector()
            .and_then(|selector| async move {
                Ok(selector
                    .bests()
                    .random()
                    .and_then(|peer| Some(&peer.peer_info))
                    .cloned())
            })
            .boxed()
    }

    /// Get all peers, the peer's order is unsorted.
    fn peer_set(&self) -> BoxFuture<Result<Vec<PeerInfo>>>;

    fn get_peer(&self, peer_id: PeerId) -> BoxFuture<Result<Option<PeerInfo>>>;

    fn get_self_peer(&self) -> BoxFuture<Result<PeerInfo>>;

    fn peer_selector(&self) -> BoxFuture<Result<PeerSelector>> {
        self.peer_set()
            .and_then(|peers| async move { Ok(PeerSelector::new(peers)) })
            .boxed()
    }
}

pub struct Selector<'a> {
    peers: Vec<&'a PeerDetail>,
}

impl<'a> Selector<'a> {
    pub fn new(peers: Vec<&'a PeerDetail>) -> Self {
        Self { peers }
    }

    pub fn empty() -> Self {
        Self::new(vec![])
    }

    pub fn first(&self) -> Option<&PeerDetail> {
        self.peers.get(0).copied()
    }

    pub fn random_peer_id(&self) -> Option<PeerId> {
        self.random().map(|peer| peer.peer_info.peer_id())
    }

    pub fn random(&self) -> Option<&PeerDetail> {
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
                .filter(|peer| predicate(&peer.peer_info))
                .collect(),
        )
    }

    pub fn filter_by_block_number(self, block_number: BlockNumber) -> Selector<'a> {
        self.filter(move |peer| peer.latest_header().number >= block_number)
    }

    pub fn cloned(self) -> Vec<PeerDetail> {
        self.peers.into_iter().cloned().collect()
    }

    pub fn into_selector(self) -> PeerSelector {
        PeerSelector::new_with_score(self.cloned())
    }
}

impl<'a> From<Vec<&'a PeerDetail>> for Selector<'a> {
    fn from(peers: Vec<&'a PeerDetail>) -> Self {
        Self::new(peers)
    }
}

#[derive(Clone)]
pub struct PeerDetail {
    peer_info: PeerInfo,
    score_counter: ScoreCounter,
}

impl PeerDetail {
    pub fn peer_id(&self) -> PeerId {
        self.peer_info.peer_id()
    }

    pub fn latest_header(&self) -> &BlockHeader {
        self.peer_info.latest_header()
    }

    pub fn peer_info(&self) -> &PeerInfo {
        &self.peer_info
    }
}

impl From<PeerInfo> for PeerDetail {
    fn from(peer: PeerInfo) -> Self {
        Self {
            peer_info: peer,
            score_counter: ScoreCounter::default(),
        }
    }
}

#[derive(Clone)]
pub struct PeerSelector {
    peers: Vec<PeerDetail>,
}

impl PeerSelector {
    pub fn new(peers: Vec<PeerInfo>) -> Self {
        let peer_detail_vec = peers
            .into_iter()
            .map(|peer| -> PeerDetail { peer.into() })
            .collect();
        Self::new_with_score(peer_detail_vec)
    }

    pub fn new_with_score(peers: Vec<PeerDetail>) -> Self {
        Self { peers }
    }

    pub fn fork(&self, peers: Vec<PeerId>) -> Self {
        let details = self
            .peers
            .iter()
            .filter(|peer| peers.contains(&peer.peer_id()))
            .map(|peer| peer.clone())
            .collect();
        Self::new_with_score(details)
    }

    /// Get top N peers sorted by total_difficulty
    pub fn top(self, n: usize) -> Self {
        if self.is_empty() {
            return self;
        }
        let mut peers = self.peers;
        Self::sort(&mut peers);
        Self::new_with_score(peers.into_iter().take(n).collect())
    }

    pub fn peer_score(&self, peer_id: &PeerId, score: i64) {
        self.peers
            .iter()
            .filter(|peer| &peer.peer_id() == peer_id)
            .for_each(|peer| peer.score_counter.inc_by(score));
    }

    /// Filter by the max total_difficulty
    pub fn bests(&self) -> Selector {
        if self.is_empty() {
            return Selector::empty();
        }
        let peers: Vec<&PeerDetail> = vec![];
        let best_peers = self
            .peers
            .iter()
            .sorted_by_key(|peer| peer.peer_info.total_difficulty())
            .rev()
            .fold(peers, |mut peers, peer| {
                if peers.is_empty()
                    || peer.peer_info.total_difficulty() >= peers[0].peer_info.total_difficulty()
                {
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
            .map(|peer| peer.peer_info.peer_id())
    }

    pub fn random(&self) -> Option<&PeerInfo> {
        self.peers
            .iter()
            .choose(&mut rand::thread_rng())
            .and_then(|peer| Some(&peer.peer_info))
    }

    pub fn first(&self) -> Option<&PeerInfo> {
        self.peers.get(0).and_then(|peer| Some(&peer.peer_info))
    }

    pub fn is_empty(&self) -> bool {
        self.peers.is_empty()
    }

    pub fn len(&self) -> usize {
        self.peers.len()
    }

    fn sort(peers: &mut Vec<PeerDetail>) {
        peers.sort_by_key(|p| p.peer_info.total_difficulty());
        peers.reverse();
    }
}

impl IntoIterator for PeerSelector {
    type Item = PeerDetail;
    type IntoIter = std::vec::IntoIter<PeerDetail>;

    fn into_iter(self) -> Self::IntoIter {
        self.peers.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use crate::peer_provider::PeerSelector;
    use starcoin_crypto::HashValue;
    use starcoin_types::block::BlockHeader;
    use starcoin_types::peer_info::{PeerId, PeerInfo};
    use starcoin_types::startup_info::{ChainInfo, ChainStatus};

    #[test]
    fn test_peer_selector() {
        let peers = vec![
            PeerInfo::new(
                PeerId::random(),
                ChainInfo::new(
                    1.into(),
                    HashValue::zero(),
                    ChainStatus::new(BlockHeader::random(), 100.into()),
                ),
            ),
            PeerInfo::new(
                PeerId::random(),
                ChainInfo::new(
                    1.into(),
                    HashValue::zero(),
                    ChainStatus::new(BlockHeader::random(), 99.into()),
                ),
            ),
            PeerInfo::new(
                PeerId::random(),
                ChainInfo::new(
                    1.into(),
                    HashValue::zero(),
                    ChainStatus::new(BlockHeader::random(), 100.into()),
                ),
            ),
            PeerInfo::new(
                PeerId::random(),
                ChainInfo::new(
                    1.into(),
                    HashValue::zero(),
                    ChainStatus::new(BlockHeader::random(), 1.into()),
                ),
            ),
        ];

        let peer_selector = PeerSelector::new(peers);
        let beat_selector = peer_selector.bests();
        assert_eq!(2, beat_selector.len());

        let top_selector = peer_selector.top(3);
        assert_eq!(3, top_selector.len());
    }
}
