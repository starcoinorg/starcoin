// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::peer_score::ScoreCounter;
use crate::PeerId;
use crate::PeerInfo;
use anyhow::Result;
use futures::future::BoxFuture;
use futures::{FutureExt, TryFutureExt};
use itertools::Itertools;
use parking_lot::Mutex;
use rand::prelude::IteratorRandom;
use rand::prelude::SliceRandom;
use rand::Rng;
use serde::{Deserialize, Serialize};
use starcoin_types::block::BlockHeader;
use std::fmt::{Debug, Formatter};
use std::sync::atomic::{AtomicU64, Ordering};
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
            .and_then(|peers| async move { Ok(PeerSelector::new(peers, PeerStrategy::default())) })
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

    pub fn score(&self) -> u64 {
        self.score_counter.score()
    }

    pub fn avg_score(&self) -> u64 {
        self.score_counter.avg()
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

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum PeerStrategy {
    Random,
    WeightedRandom,
    Best,
    Avg,
}

impl Default for PeerStrategy {
    fn default() -> Self {
        PeerStrategy::WeightedRandom
    }
}

impl std::fmt::Display for PeerStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let display = match self {
            Self::Random => "random",
            Self::WeightedRandom => "weighted",
            Self::Best => "top",
            Self::Avg => "avg",
        };
        write!(f, "{}", display)
    }
}

impl std::str::FromStr for PeerStrategy {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use self::PeerStrategy::*;

        match s {
            "random" => Ok(Random),
            "weighted" => Ok(WeightedRandom),
            "top" => Ok(Best),
            "avg" => Ok(Avg),
            other => Err(format!("Unknown peer strategy: {}", other)),
        }
    }
}

#[derive(Clone)]
pub struct PeerSelector {
    details: Arc<Mutex<Vec<PeerDetail>>>,
    total_score: Arc<AtomicU64>,
    strategy: PeerStrategy,
}

impl Debug for PeerSelector {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "peer len : {:?}, strategy : {:?}, total score : {:?}",
            self.details.lock().len(),
            self.strategy,
            self.total_score.load(Ordering::SeqCst)
        )
    }
}

impl PeerSelector {
    pub fn new(peers: Vec<PeerInfo>, strategy: PeerStrategy) -> Self {
        let len = peers.len() as u64;
        let peer_details = peers
            .into_iter()
            .map(|peer| -> PeerDetail { peer.into() })
            .collect();
        Self {
            details: Arc::new(Mutex::new(peer_details)),
            total_score: Arc::new(AtomicU64::new(len)),
            strategy,
        }
    }

    pub fn peer_info(&self, peer_id: &PeerId) -> Option<PeerInfo> {
        self.details
            .lock()
            .iter()
            .find(|peer| &peer.peer_id() == peer_id)
            .map(|peer| peer.peer_info.clone())
    }

    /// Get top N peers sorted by total_difficulty
    pub fn top(self, n: usize) -> Vec<PeerId> {
        if self.is_empty() {
            return Vec::new();
        }
        let mut peers = self.details.lock();
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
        self.details
            .lock()
            .iter()
            .filter(|peer| &peer.peer_id() == peer_id)
            .for_each(|peer| peer.score_counter.inc_by(score));
        self.total_score.fetch_add(score as u64, Ordering::SeqCst);
    }

    fn peer_exist(&self, peer_id: &PeerId) -> bool {
        for peer in self.details.lock().iter() {
            if &peer.peer_id() == peer_id {
                return true;
            }
        }
        false
    }

    pub fn add_peer(&self, peer_info: PeerInfo) {
        if !self.peer_exist(&peer_info.peer_id()) {
            self.details.lock().push(peer_info.into());
        }
    }

    /// Filter by the max total_difficulty
    pub fn bests(&self) -> Option<Vec<PeerInfo>> {
        if self.is_empty() {
            return None;
        }
        let peers: Vec<PeerInfo> = vec![];
        let best_peers = self
            .details
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
        self.details
            .lock()
            .iter()
            .map(|peer| peer.peer_info().clone())
            .collect()
    }

    pub fn retain(&self, peers: &[PeerId]) {
        let mut score: u64 = 0;
        self.details.lock().retain(|peer| -> bool {
            let flag = peers.contains(&peer.peer_id());
            if flag {
                score += peer.score_counter.score();
            }
            flag
        });
        self.total_score.store(score, Ordering::SeqCst);
    }

    pub fn select_peer(&self) -> Option<PeerId> {
        let avg_score = self.total_score.load(Ordering::SeqCst) / self.len() as u64;
        if avg_score < 200 {
            return self.random();
        }
        match &self.strategy {
            PeerStrategy::Random => self.random(),
            PeerStrategy::WeightedRandom => self.weighted_random(),
            PeerStrategy::Best => self.top_score(),
            PeerStrategy::Avg => self.avg_score(),
        }
    }

    pub fn random(&self) -> Option<PeerId> {
        self.details
            .lock()
            .iter()
            .choose(&mut rand::thread_rng())
            .map(|peer| peer.peer_info.peer_id())
    }

    pub fn top_score(&self) -> Option<PeerId> {
        if self.is_empty() {
            return None;
        }

        let lock = self.details.lock();
        let mut top_score_peer = lock.get(0).expect("Peer details is none.");
        lock.iter().for_each(|peer| {
            if peer.score() > top_score_peer.score() {
                top_score_peer = peer;
            }
        });

        Some(top_score_peer.peer_id())
    }

    pub fn avg_score(&self) -> Option<PeerId> {
        if self.is_empty() {
            return None;
        }

        let lock = self.details.lock();
        let mut top_score_peer = lock.get(0).expect("Peer details is none.");
        lock.iter().for_each(|peer| {
            if peer.avg_score() > top_score_peer.avg_score() {
                top_score_peer = peer;
            }
        });

        Some(top_score_peer.peer_id())
    }

    pub fn weighted_random(&self) -> Option<PeerId> {
        if self.is_empty() {
            return None;
        }

        if self.len() == 1 {
            return self.details.lock().get(0).map(|peer| peer.peer_id());
        }

        let mut random = rand::thread_rng();
        let total_score = self.total_score.load(Ordering::SeqCst);
        let random_score: u64 = random.gen_range(1, total_score);
        let mut tmp_score: u64 = 0;
        for peer_detail in self.details.lock().iter() {
            tmp_score += peer_detail.score_counter.score();
            if tmp_score > random_score {
                return Some(peer_detail.peer_id());
            }
        }
        None
    }

    pub fn random_peer(&self) -> Option<PeerInfo> {
        self.details
            .lock()
            .iter()
            .choose(&mut rand::thread_rng())
            .map(|peer| peer.peer_info.clone())
    }

    pub fn first_peer(&self) -> Option<PeerInfo> {
        self.details
            .lock()
            .get(0)
            .map(|peer| peer.peer_info.clone())
    }

    pub fn is_empty(&self) -> bool {
        self.details.lock().is_empty()
    }

    pub fn len(&self) -> usize {
        self.details.lock().len()
    }

    fn sort(peers: &mut Vec<PeerDetail>) {
        peers.sort_by_key(|p| p.peer_info.total_difficulty());
        peers.reverse();
    }

    pub fn scores(&self) -> Vec<(PeerId, u64)> {
        self.details
            .lock()
            .iter()
            .map(|peer| (peer.peer_id(), peer.score_counter.score()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::peer_provider::{PeerSelector, PeerStrategy};
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

        let peer_selector = PeerSelector::new(peers, PeerStrategy::default());
        let beat_selector = peer_selector.bests().unwrap();
        assert_eq!(2, beat_selector.len());

        let top_selector = peer_selector.top(3);
        assert_eq!(3, top_selector.len());
    }
}
