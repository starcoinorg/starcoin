// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::peer_provider::{PeerSelector, PeerStrategy};
use crate::peer_score::{InverseScore, Score};
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::*;
use starcoin_types::peer_info::{PeerId, PeerInfo};
use starcoin_types::startup_info::{ChainInfo, ChainStatus};
use starcoin_types::U256;

#[test]
fn test_inverse_score() {
    let inverse_score = InverseScore::new(100, 90);
    let mut score = inverse_score.execute(50);
    info!("{:?}", score);
    assert!(score > 90);
    score = inverse_score.execute(100);
    info!("{:?}", score);
    assert_eq!(score, 90);
    score = inverse_score.execute(200);
    info!("{:?}", score);
    assert!(score < 90);
}

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
            vec![],
            vec![],
        ),
        PeerInfo::new(
            PeerId::random(),
            ChainInfo::new(1.into(), HashValue::zero(), mock_chain_status(99.into())),
            vec![],
            vec![],
        ),
        PeerInfo::new(
            PeerId::random(),
            ChainInfo::new(1.into(), HashValue::zero(), mock_chain_status(100.into())),
            vec![],
            vec![],
        ),
        PeerInfo::new(
            PeerId::random(),
            ChainInfo::new(1.into(), HashValue::zero(), mock_chain_status(1.into())),
            vec![],
            vec![],
        ),
    ];

    let peer_selector = PeerSelector::new(peers, PeerStrategy::default(), None);
    let best_selector = peer_selector.bests(0.into()).unwrap();
    assert_eq!(2, best_selector.len());

    let top_selector = peer_selector.top(3);
    assert_eq!(3, top_selector.len());
}

#[test]
fn test_better_peer() {
    let mut peers = Vec::new();
    for _ in 0..20 {
        peers.push(PeerInfo::random());
    }

    let first_peer = peers.get(0).cloned().expect("first peer must exist.");

    let peer_selector = PeerSelector::new(peers, PeerStrategy::default(), None);
    let better_selector = peer_selector.betters(first_peer.total_difficulty(), 10);
    if let Some(better_selector) = better_selector {
        assert!(!better_selector.contains(&first_peer));
        better_selector.iter().for_each(|better_peer| {
            assert!(better_peer.total_difficulty() >= first_peer.total_difficulty());
        });
    } else {
        peer_selector
            .bests(0.into())
            .unwrap()
            .iter()
            .for_each(|peer| {
                assert!(peer.total_difficulty() <= first_peer.total_difficulty());
            })
    }
}
