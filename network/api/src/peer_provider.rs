// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::PeerId;
use crate::PeerInfo;
use anyhow::{format_err, Result};
use futures::future::BoxFuture;
use futures::{FutureExt, TryFutureExt};
use rand::prelude::IteratorRandom;
use starcoin_types::block::BlockNumber;

pub trait PeerProvider {
    fn identify(&self) -> PeerId;

    fn best_peer(&self) -> BoxFuture<Result<Option<PeerInfo>>> {
        self.peer_selector()
            .and_then(|selector| async move { Ok(selector.bests().random()) })
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
    pub fn bests(self) -> Self {
        if self.is_empty() {
            return self;
        }
        let mut peers = self.peers;
        Self::sort(&mut peers);
        let max_total_difficulty = peers[0].total_difficulty;
        let best_peers = peers
            .into_iter()
            .take_while(|peer| peer.total_difficulty == max_total_difficulty)
            .collect();
        Self::new(best_peers)
    }

    pub fn filter<P>(self, predicate: P) -> Self
    where
        P: Fn(&PeerInfo) -> bool + Send + 'static,
    {
        if self.is_empty() {
            return self;
        }
        Self::new(
            self.peers
                .into_iter()
                .filter(|peer| predicate(peer))
                .collect(),
        )
    }

    pub fn filter_by_block_number(self, block_number: BlockNumber) -> Self {
        self.filter(move |peer| peer.latest_header.number >= block_number)
    }

    pub fn random_peer_id(&self) -> Option<PeerId> {
        self.peers
            .iter()
            .choose(&mut rand::thread_rng())
            .map(|info| info.peer_id.clone())
    }

    pub fn random(&self) -> Option<PeerInfo> {
        self.peers.iter().choose(&mut rand::thread_rng()).cloned()
    }

    pub fn first(&self) -> Option<PeerId> {
        self.peers.get(0).map(|info| info.peer_id.clone())
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
        let beat_selector = peer_selector.clone().bests();
        assert_eq!(2, beat_selector.len());

        let top_selector = peer_selector.top(3);
        assert_eq!(3, top_selector.len());
    }
}
