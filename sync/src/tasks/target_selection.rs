// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Choosing what to sync toward: the best advertised target and the bounded rounds on the way.

use crate::tasks::{BlockFetcher, BlockIdFetcher, BlockInfoFetcher, PeerOperator};
use crate::verified_rpc_client::SyncTargetMismatch;
use anyhow::{Error, Result};
use futures::future::BoxFuture;
use futures::FutureExt;
use network_api::{PeerId, PeerInfo};
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::*;
use starcoin_sync_api::SyncTarget;
use starcoin_types::block::{BlockIdAndNumber, BlockInfo, BlockNumber};
use starcoin_types::startup_info::ChainStatus;
use starcoin_types::U256;
use std::sync::Arc;

pub(super) const MAX_BETTER_PEER_SIZE: u64 = 20;

pub trait SyncFetcher: PeerOperator + BlockIdFetcher + BlockFetcher + BlockInfoFetcher {
    fn get_best_target(
        &self,
        min_head_number: BlockNumber,
        min_difficulty: U256,
    ) -> Result<Option<SyncTarget>> {
        if let Some(best_peers) = self
            .peer_selector()
            .bests_by_rank(min_head_number, min_difficulty)
        {
            //TODO fast verify best peers by accumulator
            let mut chain_statuses: Vec<(ChainStatus, Vec<PeerId>)> = best_peers
                .into_iter()
                .filter(|peer| {
                    let ok = status_is_consistent(peer.chain_info().status());
                    if !ok {
                        warn!(
                            "[sync] skip peer {} as target, its chain status is inconsistent",
                            peer.peer_id()
                        );
                    }
                    ok
                })
                .fold(vec![], |mut chain_statuses, peer| {
                    let update = chain_statuses
                        .iter_mut()
                        .find(|(chain_status, _peers)| peer.chain_info().status() == chain_status)
                        .map(|(_chain_status, peers)| {
                            peers.push(peer.peer_id());
                            true
                        })
                        .unwrap_or(false);

                    if !update {
                        chain_statuses
                            .push((peer.chain_info().status().clone(), vec![peer.peer_id()]))
                    }
                    chain_statuses
                });
            //if all best peers block info is same, block_infos len should been 1, other use majority peers block_info
            if chain_statuses.len() > 1 {
                chain_statuses.sort_by(|(_chain_status_1, peers_1), (_chain_status_2, peers_2)| {
                    peers_1.len().cmp(&peers_2.len())
                });
            }
            let Some((chain_status, peers)) = chain_statuses.pop() else {
                return Ok(None);
            };
            let header = chain_status.head;
            Ok(Some(SyncTarget {
                target_id: BlockIdAndNumber::new(header.id(), header.number()),
                block_info: chain_status.info,
                peers,
            }))
        } else {
            debug!(
                "get_best_target return None, total_peers_in_selector: {}, min_difficulty: {}",
                self.peer_selector().len(),
                min_difficulty
            );
            Ok(None)
        }
    }

    fn get_better_target(
        &self,
        min_head_number: BlockNumber,
        min_difficulty: U256,
        best_target: SyncTarget,
        max_peers: u64,
        begin_number: u64,
    ) -> BoxFuture<Result<SyncTarget>> {
        let fut = async move {
            if let Some(mut better_peers) = self.peer_selector().betters_by_rank(
                min_head_number,
                min_difficulty,
                MAX_BETTER_PEER_SIZE.saturating_mul(2),
            ) {
                let selector = self.peer_selector();
                better_peers.sort_by(|info_1, info_2| {
                    selector
                        .rank_for(info_1.block_number(), info_1.total_difficulty())
                        .cmp(&selector.rank_for(info_2.block_number(), info_2.total_difficulty()))
                });

                let mut peers = Vec::new();
                let mut target: Option<(BlockInfo, BlockIdAndNumber)> = None;
                for better_peer in better_peers.iter() {
                    if peers.len() >= max_peers as usize {
                        break;
                    }

                    let mut eligible = false;
                    match target.as_ref() {
                        None => {
                            let maybe_target_number = std::cmp::min(
                                begin_number.saturating_add(MAX_SUB_TARGET_SPAN),
                                better_peer.block_number(),
                            );
                            if maybe_target_number == better_peer.block_number()
                                && best_target.peers.contains(&better_peer.peer_id())
                            {
                                target = Some((
                                    better_peer.chain_info.status().info().clone(),
                                    BlockIdAndNumber {
                                        number: better_peer.latest_header().number(),
                                        id: better_peer.latest_header().id(),
                                    },
                                ));
                                eligible = true;
                            } else if let Some(block_id) = self
                                .fetch_block_id(
                                    best_target.peers.first().cloned(),
                                    maybe_target_number,
                                )
                                .await?
                            {
                                let mut block_info = None;
                                if block_id == better_peer.block_id() {
                                    block_info =
                                        Some(better_peer.chain_info.status().info().clone());
                                } else if let Some(better_block_id) = self
                                    .fetch_block_id(
                                        Some(better_peer.peer_id()),
                                        maybe_target_number,
                                    )
                                    .await
                                    .unwrap_or_else(|e| skip_peer(better_peer, e))
                                {
                                    if block_id == better_block_id {
                                        block_info = self
                                            .fetch_block_info(Some(better_peer.peer_id()), block_id)
                                            .await
                                            .unwrap_or_else(|e| skip_peer(better_peer, e));
                                    }
                                }

                                // The info is the peer's word; it must describe the block
                                // we asked about, or its leaf count can make the accumulator
                                // phase pull ids without bound.
                                if let Some(info) = block_info.filter(|info| {
                                    info_describes(info, block_id, maybe_target_number)
                                }) {
                                    eligible = true;
                                    target = Some((
                                        info,
                                        BlockIdAndNumber {
                                            number: maybe_target_number,
                                            id: block_id,
                                        },
                                    ));
                                }
                            }
                        }
                        Some((_, target_id_number)) => {
                            if best_target.peers.contains(&better_peer.peer_id()) {
                                eligible = true;
                            } else if let Some(block_id) = self
                                .fetch_block_id(
                                    Some(better_peer.peer_id()),
                                    target_id_number.number,
                                )
                                .await
                                .unwrap_or_else(|e| skip_peer(better_peer, e))
                            {
                                if block_id == target_id_number.id {
                                    eligible = true;
                                }
                            }
                        }
                    }

                    if eligible {
                        peers.push(better_peer.peer_id());
                    }
                }

                if let Some((block_info, id_number)) = target {
                    return Ok(SyncTarget {
                        target_id: id_number,
                        block_info,
                        peers,
                    });
                }
            }
            self.step_toward(best_target, begin_number).await
        };

        fut.boxed()
    }

    /// With no better peer in reach, sync toward `best_target` in the same bounded rounds as the
    /// better-peer path, using block info fetched for the round's own end block.
    fn step_toward(
        &self,
        best_target: SyncTarget,
        begin_number: BlockNumber,
    ) -> BoxFuture<Result<SyncTarget>> {
        let fut = async move {
            let number = begin_number.saturating_add(MAX_SUB_TARGET_SPAN);
            if best_target.target_id.number() <= number {
                return Ok(best_target);
            }
            // Only this peer vouches for the round's block info, so only it takes the round
            // and the blame if the info turns out wrong.
            let vouching_peers: Vec<PeerId> =
                best_target.peers.first().cloned().into_iter().collect();
            let peer = vouching_peers.first().cloned();
            let block_info = match self.fetch_block_id(peer.clone(), number).await? {
                Some(id) => self
                    .fetch_block_info(peer, id)
                    .await?
                    .filter(|info| info_describes(info, id, number)),
                None => None,
            };
            match block_info {
                Some(block_info) => Ok(SyncTarget {
                    target_id: BlockIdAndNumber {
                        number,
                        id: block_info.block_id,
                    },
                    block_info,
                    peers: vouching_peers,
                }),
                None => Err(SyncTargetMismatch {
                    peers: vouching_peers,
                    msg: format!("target peer cannot describe block {}", number),
                }
                .into()),
            }
        };
        fut.boxed()
    }
}

impl<T> SyncFetcher for Arc<T> where T: SyncFetcher {}

/// Blocks per sync round. Each round's accumulator phase takes at most this many ids before
/// the peer has to deliver real, verified blocks.
const MAX_SUB_TARGET_SPAN: u64 = 1000;

fn info_describes(info: &BlockInfo, id: HashValue, number: BlockNumber) -> bool {
    info.block_id == id && info.block_accumulator_info.num_leaves == number.saturating_add(1)
}

/// A peer's advertised head and block info must describe the same block. Nothing in the
/// handshake or compact block path proves the advertised total difficulty, so a mismatched
/// pair is at least an easy lie to drop before choosing a sync target.
pub(super) fn status_is_consistent(status: &ChainStatus) -> bool {
    status.info.block_id == status.head.id()
        && status.info.block_accumulator_info.num_leaves == status.head.number().saturating_add(1)
}

/// One peer failing to answer must not abort the search for a better target.
fn skip_peer<T>(peer: &PeerInfo, err: Error) -> Option<T> {
    warn!("[sync] skip better peer {}: {:?}", peer.peer_id(), err);
    None
}

#[cfg(test)]
mod status_tests {
    use super::status_is_consistent;
    use starcoin_types::startup_info::ChainStatus;

    fn consistent() -> ChainStatus {
        let mut s = ChainStatus::random();
        s.info.block_id = s.head.id();
        s.info.block_accumulator_info.num_leaves = s.head.number().saturating_add(1);
        s
    }

    #[test]
    fn head_and_info_must_describe_the_same_block() {
        assert!(status_is_consistent(&consistent()));
        let mut other_block = consistent();
        other_block.info.block_id = starcoin_crypto::HashValue::random();
        assert!(!status_is_consistent(&other_block));
        let mut fake_height = consistent();
        fake_height.info.block_accumulator_info.num_leaves = fake_height
            .info
            .block_accumulator_info
            .num_leaves
            .saturating_add(1_000_000);
        assert!(!status_is_consistent(&fake_height));
    }
}
