// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::peer_score::{PeerScoreMetrics, ScoreCounter};
use crate::PeerId;
use crate::PeerInfo;
use anyhow::Result;
use futures::channel::oneshot::Receiver;
use futures::future::BoxFuture;
use itertools::Itertools;
use network_p2p_types::ReputationChange;
use parking_lot::Mutex;
use rand::prelude::IteratorRandom;
use rand::prelude::SliceRandom;
use rand::Rng;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use starcoin_types::block::BlockHeader;
use starcoin_types::U256;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

pub trait PeerProvider: Send + Sync + std::marker::Unpin {
    /// Get all peers, the peer's order is unsorted.
    fn peer_set(&self) -> BoxFuture<Result<Vec<PeerInfo>>>;

    fn get_peer(&self, peer_id: PeerId) -> BoxFuture<Result<Option<PeerInfo>>>;

    fn get_self_peer(&self) -> BoxFuture<Result<PeerInfo>>;

    fn report_peer(&self, peer_id: PeerId, cost_benefit: ReputationChange);

    fn reputations(
        &self,
        reputation_threshold: i32,
    ) -> BoxFuture<'_, Result<Receiver<Vec<(PeerId, i32)>>>>;

    fn ban_peer(&self, peer_id: PeerId, ban: bool);
}

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

impl From<(PeerInfo, u64)> for PeerDetail {
    fn from(peer: (PeerInfo, u64)) -> Self {
        Self {
            peer_info: peer.0,
            score_counter: ScoreCounter::new(peer.1),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize, JsonSchema)]
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
    peer_score_metrics: Option<PeerScoreMetrics>,
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
    pub fn new(
        peers: Vec<PeerInfo>,
        strategy: PeerStrategy,
        peer_score_metrics: Option<PeerScoreMetrics>,
    ) -> Self {
        Self::new_with_reputation(Vec::new(), peers, strategy, peer_score_metrics)
    }

    pub fn new_with_reputation(
        reputations: Vec<(PeerId, u64)>,
        peers: Vec<PeerInfo>,
        strategy: PeerStrategy,
        peer_score_metrics: Option<PeerScoreMetrics>,
    ) -> Self {
        let reputations = reputations.into_iter().collect::<HashMap<PeerId, u64>>();
        let mut total_score = 0u64;
        let peer_details = peers
            .into_iter()
            .map(|peer| -> PeerDetail {
                let score = if let Some(reputation) = reputations.get(&peer.peer_id()) {
                    *reputation
                } else {
                    1
                };
                total_score = total_score.saturating_add(score);
                (peer, score).into()
            })
            .collect();
        Self {
            details: Arc::new(Mutex::new(peer_details)),
            total_score: Arc::new(AtomicU64::new(total_score)),
            strategy,
            peer_score_metrics,
        }
    }

    pub fn switch_strategy(&mut self, strategy: PeerStrategy) {
        self.strategy = strategy
    }

    pub fn strategy(&self) -> PeerStrategy {
        self.strategy
    }

    pub fn peer_info(&self, peer_id: &PeerId) -> Option<PeerInfo> {
        self.details
            .lock()
            .iter()
            .find(|peer| &peer.peer_id() == peer_id)
            .map(|peer| peer.peer_info.clone())
    }

    /// Get top N peers sorted by total_difficulty
    pub fn top(&self, n: usize) -> Vec<PeerId> {
        if self.is_empty() {
            return Vec::new();
        }
        self.details
            .lock()
            .iter()
            .sorted_by_key(|peer| peer.peer_info.total_difficulty())
            .rev()
            .map(|peer| peer.peer_info.peer_id())
            .take(n)
            .collect()
    }

    //TODO performance review
    pub fn peer_score(&self, peer_id: &PeerId, score: u64) {
        self.details
            .lock()
            .iter()
            .find(|peer| &peer.peer_id() == peer_id)
            .map(|peer| {
                peer.score_counter.inc_by(score);
                Some(peer)
            });
        let total_score = self.total_score.fetch_add(score, Ordering::SeqCst);
        if let Some(peer_score_metrics) = self.peer_score_metrics.as_ref() {
            peer_score_metrics
                .peer_score
                .with_label_values(&[format!("{}", peer_id).as_str()])
                .set(score);
            peer_score_metrics.total_score.set(total_score);
        }
    }

    pub fn peer_exist(&self, peer_id: &PeerId) -> bool {
        for peer in self.details.lock().iter() {
            if &peer.peer_id() == peer_id {
                return true;
            }
        }
        false
    }

    pub fn add_or_update_peer(&self, peer_info: PeerInfo) {
        let mut details = self.details.lock();
        let update = details
            .iter_mut()
            .find(|peer| peer.peer_id() == peer_info.peer_id())
            .map(|peer| {
                peer.peer_info = peer_info.clone();
                true
            })
            .unwrap_or(false);
        if !update {
            details.push(peer_info.into())
        }
    }

    /// Filter by the max total_difficulty
    pub fn bests(&self, min_difficulty: U256) -> Option<Vec<PeerInfo>> {
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
        if best_peers.is_empty() || best_peers[0].total_difficulty() <= min_difficulty {
            None
        } else {
            Some(best_peers)
        }
    }

    pub fn betters(&self, difficulty: U256, max_peers: u64) -> Option<Vec<PeerInfo>> {
        if self.is_empty() {
            return None;
        }
        let betters: Vec<PeerInfo> = self
            .details
            .lock()
            .iter()
            .filter(|peer| peer.peer_info().total_difficulty() > difficulty)
            .sorted_by(|peer_1, peer_2| Ord::cmp(&peer_2.score(), &peer_1.score()))
            .take(max_peers as usize)
            .map(|peer| peer.peer_info().clone())
            .collect();
        if betters.is_empty() {
            None
        } else {
            Some(betters)
        }
    }

    pub fn best(&self) -> Option<PeerInfo> {
        if let Some(peers) = self.bests(0.into()) {
            peers.choose(&mut rand::thread_rng()).cloned()
        } else {
            None
        }
    }

    pub fn peers(&self) -> Vec<PeerId> {
        self.details
            .lock()
            .iter()
            .map(|peer| peer.peer_id())
            .collect()
    }

    pub fn peers_by_filter<F>(&self, f: F) -> Vec<PeerId>
    where
        F: Fn(&PeerInfo) -> bool,
    {
        self.details
            .lock()
            .iter()
            .filter(|peer| f(peer.peer_info()))
            .map(|peer| peer.peer_id())
            .collect()
    }

    pub fn retain(&self, peers: &[PeerId]) {
        self.retain_by_filter(|peer| peers.contains(&peer.peer_id()));
    }

    pub fn retain_by_filter<F>(&self, filter: F)
    where
        F: Fn(&PeerDetail) -> bool,
    {
        let mut score: u64 = 0;
        self.details.lock().retain(|peer| -> bool {
            let flag = filter(peer);
            if flag {
                score = score.saturating_add(peer.score_counter.score());
            }
            flag
        });
        self.total_score.store(score, Ordering::SeqCst);
    }

    /// Retain the peer which supported rpc call.
    pub fn retain_rpc_peers(&self) {
        self.retain_by_filter(|peer| peer.peer_info.is_support_rpc())
    }

    /// Retain the peer which supported rpc call.
    pub fn retain_rpc_peers_by_protocol(&self, protocols: &[Cow<'static, str>]) {
        self.retain_by_filter(move |peer| peer.peer_info.is_support_rpc_protocols(protocols))
    }

    pub fn remove_peer(&self, peer: &PeerId) -> usize {
        let mut lock = self.details.lock();
        for (index, p) in lock.iter().enumerate() {
            if &p.peer_id() == peer {
                let detail = lock.remove(index);
                let score = self.total_score.load(Ordering::SeqCst);
                self.total_score
                    .store(score.saturating_sub(detail.score()), Ordering::SeqCst);
                break;
            }
        }

        lock.len()
    }

    pub fn select_peer(&self) -> Option<PeerId> {
        let avg_score = self
            .total_score
            .load(Ordering::SeqCst)
            .checked_div(self.len() as u64)?;
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

    fn top_one<F>(&self, cmp: F) -> Option<PeerId>
    where
        F: Fn(&PeerDetail, &PeerDetail) -> bool,
    {
        let lock = self.details.lock();
        let mut iter = lock.iter();
        let first = iter.next()?;
        let top = iter.fold(
            first,
            |top, current| {
                if cmp(top, current) {
                    top
                } else {
                    current
                }
            },
        );
        Some(top.peer_id())
    }

    pub fn top_score(&self) -> Option<PeerId> {
        self.top_one(|top, current| top.score() >= current.score())
    }

    pub fn avg_score(&self) -> Option<PeerId> {
        self.top_one(|top, current| top.avg_score() >= current.avg_score())
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
        let random_score: u64 = random.gen_range(1..total_score);
        let mut tmp_score: u64 = 0;
        for peer_detail in self.details.lock().iter() {
            tmp_score = tmp_score.saturating_add(peer_detail.score_counter.score());
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

    pub fn scores(&self) -> Vec<(PeerId, u64)> {
        self.details
            .lock()
            .iter()
            .map(|peer| (peer.peer_id(), peer.score_counter.score()))
            .collect()
    }
}
