use anyhow::{format_err, Result};
use futures::executor::block_on;
use futures::future::BoxFuture;
use futures::FutureExt;
use network_api::{PeerId, PeerInfo, PeerSelector, PeerStrategy};
use network_p2p_core::{NetRpcError, RawRpcClient};
use network_p2p_types::{OutboundFailure, RequestFailure};
use starcoin_chain_api::ChainAsyncService;
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::*;
use starcoin_network_rpc_api::GetBlockIds;
use starcoin_sync::verified_rpc_client::VerifiedRpcClient;
use starcoin_types::block::Block;
use std::borrow::Cow;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use tokio::runtime::Runtime;

struct MockRpcClient {
    peer_id1: PeerId, // This peer will always fail
    peer_id2: PeerId, // This peer will succeed
    blocks: Arc<Mutex<Vec<Block>>>,
    call_count: Arc<Mutex<usize>>,
}

impl MockRpcClient {
    fn new(peer_id1: PeerId, peer_id2: PeerId, blocks: Vec<Block>) -> Self {
        Self {
            peer_id1,
            peer_id2,
            blocks: Arc::new(Mutex::new(blocks)),
            call_count: Arc::new(Mutex::new(0)),
        }
    }
}

impl RawRpcClient for MockRpcClient {
    fn send_raw_request(
        &self,
        peer_id: PeerId,
        _rpc_path: Cow<'static, str>,
        message: Vec<u8>,
    ) -> BoxFuture<Result<Vec<u8>>> {
        *self.call_count.lock().unwrap() += 1;
        if peer_id == self.peer_id1 {
            futures::future::ready(Err(format_err!("NotConnected"))).boxed()
        } else if peer_id == self.peer_id2 {
            let blocks = self.blocks.lock().unwrap();
            let request_ids: Vec<HashValue> = bcs_ext::from_bytes(&message).unwrap();
            let response_blocks: Vec<Option<Block>> = request_ids
                .iter()
                .map(|id| blocks.iter().find(|b| &b.id() == id).cloned())
                .collect();
            let data_bytes = bcs_ext::to_bytes(&response_blocks).unwrap();
            let rpc_result: network_p2p_core::Result<Vec<u8>, NetRpcError> = Ok(data_bytes);
            let response_bytes = bcs_ext::to_bytes(&rpc_result).unwrap();
            futures::future::ready(Ok(response_bytes)).boxed()
        } else {
            futures::future::ready(Err(format_err!("Unknown peer"))).boxed()
        }
    }
}

struct TimeoutRpcClient;

impl RawRpcClient for TimeoutRpcClient {
    fn send_raw_request(
        &self,
        _peer_id: PeerId,
        _rpc_path: Cow<'static, str>,
        _message: Vec<u8>,
    ) -> BoxFuture<Result<Vec<u8>>> {
        futures::future::pending().boxed()
    }
}

#[derive(Clone, Copy)]
enum AdaptiveReplyPlan {
    Timeout,
    NonTimeoutError,
    Success,
}

#[derive(Clone)]
struct AdaptiveGetBlockIdsRpcClient {
    plans: Arc<Mutex<Vec<AdaptiveReplyPlan>>>,
    observed_max_sizes: Arc<Mutex<Vec<u64>>>,
}

impl AdaptiveGetBlockIdsRpcClient {
    fn new(plans: Vec<AdaptiveReplyPlan>) -> Self {
        Self {
            plans: Arc::new(Mutex::new(plans)),
            observed_max_sizes: Arc::new(Mutex::new(vec![])),
        }
    }

    fn observed_max_sizes(&self) -> Vec<u64> {
        self.observed_max_sizes.lock().unwrap().clone()
    }
}

impl RawRpcClient for AdaptiveGetBlockIdsRpcClient {
    fn send_raw_request(
        &self,
        _peer_id: PeerId,
        _rpc_path: Cow<'static, str>,
        message: Vec<u8>,
    ) -> BoxFuture<Result<Vec<u8>>> {
        let req: GetBlockIds = bcs_ext::from_bytes(&message).unwrap();
        self.observed_max_sizes.lock().unwrap().push(req.max_size);

        let plan = self.plans.lock().unwrap().remove(0);

        match plan {
            AdaptiveReplyPlan::Timeout => futures::future::ready(Err(RequestFailure::Network(
                OutboundFailure::Timeout,
            )
            .into()))
            .boxed(),
            AdaptiveReplyPlan::NonTimeoutError => futures::future::ready(Err(
                RequestFailure::Network(OutboundFailure::ConnectionClosed).into(),
            ))
            .boxed(),
            AdaptiveReplyPlan::Success => {
                let ids = (0..req.max_size)
                    .map(|_| HashValue::random())
                    .collect::<Vec<_>>();
                let data_bytes = bcs_ext::to_bytes(&ids).unwrap();
                let rpc_result: network_p2p_core::Result<Vec<u8>, NetRpcError> = Ok(data_bytes);
                let response_bytes = bcs_ext::to_bytes(&rpc_result).unwrap();
                futures::future::ready(Ok(response_bytes)).boxed()
            }
        }
    }
}

fn build_single_peer_selector(peer_id: PeerId) -> PeerSelector {
    let peer_selector = PeerSelector::new(vec![], PeerStrategy::default(), None);
    let mut peer_info = PeerInfo::random();
    peer_info.peer_id = peer_id.clone();
    peer_selector.add_or_update_peer(peer_info);
    peer_selector.peer_score(&peer_id, 100);
    peer_selector
}

#[stest::test]
fn test_get_blocks_multiple_blocks_with_retry() -> Result<()> {
    let node = test_helper::run_test_node()?;
    let chain_service = node.chain_service()?;
    for _ in 0..3 {
        node.generate_block()?;
    }

    std::thread::sleep(std::time::Duration::from_millis(500));

    let head_block = block_on(async { chain_service.main_head_block().await })?;
    let parent_hash = head_block.header().parent_hash();
    let parent_block =
        block_on(async { chain_service.get_block_by_hash(parent_hash).await })?.unwrap();

    info!(
        "Head block: #{}, parent: #{}",
        head_block.header().number(),
        parent_block.header().number()
    );

    let peer_selector = PeerSelector::new(vec![], PeerStrategy::default(), None);

    let peer_id1 = PeerId::random();
    let mut peer_info1 = PeerInfo::random();
    peer_info1.peer_id = peer_id1.clone();
    peer_selector.add_or_update_peer(peer_info1);
    peer_selector.peer_score(&peer_id1, 100);
    let peer_id2 = PeerId::random();
    let mut peer_info2 = PeerInfo::random();
    peer_info2.peer_id = peer_id2.clone();
    peer_selector.add_or_update_peer(peer_info2);
    peer_selector.peer_score(&peer_id2, 99);
    let mock_client = MockRpcClient::new(
        peer_id1.clone(),
        peer_id2.clone(),
        vec![head_block.clone(), parent_block.clone()],
    );

    let verified_client = VerifiedRpcClient::new(peer_selector, mock_client, 5);

    let rt = Runtime::new()?;
    let result = rt.block_on(async {
        verified_client
            .get_blocks(vec![head_block.id(), parent_block.id()])
            .await
    })?;

    assert_eq!(result.len(), 2, "Should return 2 block results");
    assert!(result[0].is_some(), "First block should exist");
    assert!(result[1].is_some(), "Second block should exist");

    let (ret_block1, peer1) = result[0].as_ref().unwrap();
    let (ret_block2, peer2) = result[1].as_ref().unwrap();

    assert_eq!(
        ret_block1.id(),
        head_block.id(),
        "First block ID should match"
    );
    assert_eq!(
        ret_block2.id(),
        parent_block.id(),
        "Second block ID should match"
    );

    assert!(peer1.is_some(), "Peer info should be present");
    assert!(peer2.is_some(), "Peer info should be present");
    assert_eq!(
        peer1.as_ref().unwrap(),
        &peer_id2,
        "Block should come from peer_id2"
    );
    assert_eq!(
        peer2.as_ref().unwrap(),
        &peer_id2,
        "Block should come from peer_id2"
    );

    node.stop()?;
    Ok(())
}

#[stest::test]
fn test_get_blocks_timeout() -> Result<()> {
    let peer_selector = PeerSelector::new(vec![], PeerStrategy::default(), None);
    let peer_id = PeerId::random();
    let mut peer_info = PeerInfo::random();
    peer_info.peer_id = peer_id.clone();
    peer_selector.add_or_update_peer(peer_info);
    peer_selector.peer_score(&peer_id, 100);

    let verified_client = VerifiedRpcClient::new(peer_selector, TimeoutRpcClient, 1)
        .with_rpc_config(Duration::from_millis(50), 2);
    let rt = Runtime::new()?;
    let result = rt.block_on(async { verified_client.get_blocks(vec![HashValue::random()]).await });

    assert!(
        result.is_err(),
        "get_blocks should timeout and return error"
    );
    Ok(())
}

#[stest::test]
fn test_get_block_ids_adaptive_shrink_on_timeout() -> Result<()> {
    let peer_id = PeerId::random();
    let peer_selector = build_single_peer_selector(peer_id);
    let mock_client = AdaptiveGetBlockIdsRpcClient::new(vec![
        AdaptiveReplyPlan::Timeout,
        AdaptiveReplyPlan::Success,
    ]);

    let verified_client = VerifiedRpcClient::new(peer_selector, mock_client.clone(), 1)
        .with_rpc_config(Duration::from_millis(50), 3);
    let rt = Runtime::new()?;
    let _ = rt.block_on(async {
        verified_client
            .get_block_ids(None, 100, false, 10_000)
            .await
    })?;

    let sizes = mock_client.observed_max_sizes();
    assert_eq!(sizes, vec![1000, 500]);
    Ok(())
}

#[stest::test]
fn test_get_block_ids_adaptive_floor_at_ten() -> Result<()> {
    let peer_id = PeerId::random();
    let peer_selector = build_single_peer_selector(peer_id);
    let mock_client = AdaptiveGetBlockIdsRpcClient::new(vec![
        AdaptiveReplyPlan::Timeout,
        AdaptiveReplyPlan::Timeout,
        AdaptiveReplyPlan::Timeout,
        AdaptiveReplyPlan::Timeout,
        AdaptiveReplyPlan::Timeout,
        AdaptiveReplyPlan::Timeout,
        AdaptiveReplyPlan::Timeout,
        AdaptiveReplyPlan::Timeout,
        AdaptiveReplyPlan::Success,
    ]);

    let verified_client = VerifiedRpcClient::new(peer_selector, mock_client.clone(), 1)
        .with_rpc_config(Duration::from_millis(50), 10);
    let rt = Runtime::new()?;
    let _ = rt.block_on(async {
        verified_client
            .get_block_ids(None, 100, false, 10_000)
            .await
    })?;

    let sizes = mock_client.observed_max_sizes();
    assert_eq!(sizes, vec![1000, 500, 250, 125, 62, 31, 15, 10, 10]);
    Ok(())
}

#[stest::test]
fn test_get_block_ids_adaptive_grow_after_stable_success() -> Result<()> {
    let peer_id = PeerId::random();
    let peer_selector = build_single_peer_selector(peer_id);
    let mut plans = vec![AdaptiveReplyPlan::Timeout, AdaptiveReplyPlan::Success];
    for _ in 0..21 {
        plans.push(AdaptiveReplyPlan::Success);
    }
    let mock_client = AdaptiveGetBlockIdsRpcClient::new(plans);

    let verified_client = VerifiedRpcClient::new(peer_selector, mock_client.clone(), 1)
        .with_rpc_config(Duration::from_millis(50), 3);
    let rt = Runtime::new()?;

    // First call: timeout at 1000, then success at 500.
    let _ = rt.block_on(async {
        verified_client
            .get_block_ids(None, 100, false, 10_000)
            .await
    })?;
    // 19 more successful calls at 500.
    for _ in 0..19 {
        let _ = rt.block_on(async {
            verified_client
                .get_block_ids(None, 100, false, 10_000)
                .await
        })?;
    }
    // This call reaches the 10_000 stable-success threshold and triggers growth.
    let _ = rt.block_on(async {
        verified_client
            .get_block_ids(None, 100, false, 10_000)
            .await
    })?;
    // Next call should use 1000.
    let _ = rt.block_on(async {
        verified_client
            .get_block_ids(None, 100, false, 10_000)
            .await
    })?;

    let sizes = mock_client.observed_max_sizes();
    assert_eq!(sizes[0], 1000);
    assert_eq!(sizes[1], 500);
    assert_eq!(sizes[sizes.len() - 1], 1000);
    assert!(sizes.iter().skip(1).any(|size| *size == 500));
    Ok(())
}

#[stest::test]
fn test_get_block_ids_keeps_size_on_non_timeout_error() -> Result<()> {
    let peer_id = PeerId::random();
    let peer_selector = build_single_peer_selector(peer_id);
    let mock_client = AdaptiveGetBlockIdsRpcClient::new(vec![
        AdaptiveReplyPlan::NonTimeoutError,
        AdaptiveReplyPlan::Success,
    ]);

    let verified_client = VerifiedRpcClient::new(peer_selector, mock_client.clone(), 1)
        .with_rpc_config(Duration::from_millis(50), 3);
    let rt = Runtime::new()?;
    let _ = rt.block_on(async {
        verified_client
            .get_block_ids(None, 100, false, 10_000)
            .await
    })?;

    let sizes = mock_client.observed_max_sizes();
    assert_eq!(sizes, vec![1000, 1000]);
    Ok(())
}
