// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::peer_score::ScoreCounter;
use crate::PeerInfo;
use crate::{NetworkService, PeerId};
use anyhow::Result;
use futures::future::BoxFuture;
use futures::{FutureExt, TryFutureExt};
use itertools::Itertools;
use parking_lot::Mutex;
use rand::prelude::IteratorRandom;
use rand::prelude::SliceRandom;
use starcoin_types::block::{BlockHeader, BlockNumber};
use std::sync::Arc;

pub trait PeerProvider: Send + Sync {
    fn best_peer(&self) -> BoxFuture<Result<Option<PeerInfo>>> {
        self.peer_selector()
            .and_then(|selector| async move {
                Ok(if let Some(bests) = selector.bests() {
                    bests.choose(&mut rand::thread_rng()).cloned()
                } else {
                    None
                })
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
    peers: Arc<Mutex<Vec<PeerDetail>>>,
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
        Self {
            peers: Arc::new(Mutex::new(peers)),
        }
    }

    pub fn peer_info(&self, peer_id: &PeerId) -> Option<PeerInfo> {
        self.peers
            .lock()
            .iter()
            .find(|peer| &peer.peer_id() == peer_id)
            .and_then(|peer| Some(peer.peer_info.clone()))
    }

    /// Get top N peers sorted by total_difficulty
    pub fn top(self, n: usize) -> Vec<PeerId> {
        if self.is_empty() {
            return Vec::new();
        }
        let mut peers = self.peers.lock();
        Self::sort(&mut peers);
        let mut top: Vec<PeerId> = Vec::new();
        for peer in peers.iter() {
            if top.len() >= n {
                break;
            }
            top.push(peer.peer_id());
        }
        top
    }

    pub fn peer_score(&self, peer_id: &PeerId, score: i64) {
        self.peers
            .lock()
            .iter()
            .filter(|peer| &peer.peer_id() == peer_id)
            .for_each(|peer| peer.score_counter.inc_by(score));
    }

    /// Filter by the max total_difficulty
    pub fn bests(&self) -> Option<Vec<PeerInfo>> {
        if self.is_empty() {
            return None;
        }
        let peers: Vec<PeerInfo> = vec![];
        let best_peers = self
            .peers
            .lock()
            .iter()
            .sorted_by_key(|peer| peer.peer_info.total_difficulty())
            .rev()
            .fold(peers, |mut peers, peer| {
                if peers.is_empty()
                    || peer.peer_info.total_difficulty() >= peers[0].total_difficulty()
                {
                    peers.push(peer.peer_info().clone());
                };
                peers
            });
        Some(best_peers)
    }

    pub fn peers(&self) -> Vec<PeerInfo> {
        self.peers
            .lock()
            .iter()
            .map(|peer| peer.peer_info().clone())
            .collect()
    }

    pub fn retain(&self, peers: &Vec<PeerId>) {
        self.peers
            .lock()
            .retain(|peer| peers.contains(&peer.peer_id()))
    }

    pub fn random_peer_id(&self) -> Option<PeerId> {
        self.peers
            .lock()
            .iter()
            .choose(&mut rand::thread_rng())
            .map(|peer| peer.peer_info.peer_id())
    }

    pub fn random(&self) -> Option<PeerInfo> {
        self.peers
            .lock()
            .iter()
            .choose(&mut rand::thread_rng())
            .and_then(|peer| Some(peer.peer_info.clone()))
    }

    pub fn first(&self) -> Option<PeerInfo> {
        self.peers
            .lock()
            .get(0)
            .and_then(|peer| Some(peer.peer_info.clone()))
    }

    pub fn is_empty(&self) -> bool {
        self.peers.lock().is_empty()
    }

    pub fn len(&self) -> usize {
        self.peers.lock().len()
    }

    fn sort(peers: &mut Vec<PeerDetail>) {
        peers.sort_by_key(|p| p.peer_info.total_difficulty());
        peers.reverse();
    }
}

#[cfg(test)]
mod tests {
    use crate::peer_provider::PeerSelector;
    use starcoin_crypto::HashValue;
    use starcoin_types::peer_info::{PeerId, PeerInfo};
    use starcoin_types::startup_info::{ChainInfo, ChainStatus};
    use starcoin_types::U256;

    fn mock_chain_status(total_difficulty: U256) -> ChainStatus {
        let mut status = ChainStatus::random();
        status.info.total_difficulty = total_difficulty;
        status
    }
    #[test]
    fn test_peer_selector() {
        let peers = vec![
            PeerInfo::new(
                PeerId::random(),
                ChainInfo::new(1.into(), HashValue::zero(), mock_chain_status(100.into())),
            ),
            PeerInfo::new(
                PeerId::random(),
                ChainInfo::new(1.into(), HashValue::zero(), mock_chain_status(99.into())),
            ),
            PeerInfo::new(
                PeerId::random(),
                ChainInfo::new(1.into(), HashValue::zero(), mock_chain_status(100.into())),
            ),
            PeerInfo::new(
                PeerId::random(),
                ChainInfo::new(1.into(), HashValue::zero(), mock_chain_status(1.into())),
            ),
        ];

        let peer_selector = PeerSelector::new(peers);
        let beat_selector = peer_selector.bests();
        assert_eq!(2, beat_selector.len());

        let top_selector = peer_selector.top(3);
        assert_eq!(3, top_selector.len());
    }
}
