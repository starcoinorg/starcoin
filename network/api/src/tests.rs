use crate::peer_provider::{PeerSelector, PeerStrategy};
use crate::peer_score::InverseScore;
use starcoin_crypto::HashValue;
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
