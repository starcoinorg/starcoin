use anyhow::{format_err, Result};
use futures::executor::block_on;
use futures::future::BoxFuture;
use futures::FutureExt;
use network_api::{PeerId, PeerInfo, PeerSelector, PeerStrategy};
use network_p2p_core::{NetRpcError, RawRpcClient};
use starcoin_chain_api::ChainAsyncService;
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::*;
use starcoin_sync::verified_rpc_client::VerifiedRpcClient;
use starcoin_types::block::Block;
use std::borrow::Cow;
use std::sync::Arc;
use std::sync::Mutex;

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

    let result = block_on(async {
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
