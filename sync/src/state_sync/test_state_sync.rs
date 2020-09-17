use crate::helper::{
    get_accumulator_node_by_node_hash, get_state_node_by_node_hash, get_txn_infos,
};
use crate::state_sync::{
    sync_accumulator_node, sync_state_node, sync_txn_info, Inner, StateSyncTaskEvent, TxnInfoEvent,
};
use crate::sync_event_handle::SendSyncEventHandler;
use chain::BlockChain;
use config::NodeConfig;
use starcoin_accumulator::node::AccumulatorStoreType;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};
use test_helper::chain::gen_blockchain_with_blocks_for_test;
use test_helper::{gen_blockchain_for_test, DummyNetworkService};
use traits::ChainReader;
use types::peer_info::PeerId;

#[derive(Clone)]
struct TestStateSyncTaskEventHandler {
    state_count: Arc<Mutex<u64>>,
    accumulator_count: Arc<Mutex<u64>>,
    txn_info_count: Arc<Mutex<u64>>,
    event: PhantomData<StateSyncTaskEvent>,
}

impl TestStateSyncTaskEventHandler {
    fn new() -> Self {
        TestStateSyncTaskEventHandler {
            state_count: Arc::new(Mutex::new(0)),
            accumulator_count: Arc::new(Mutex::new(0)),
            txn_info_count: Arc::new(Mutex::new(0)),
            event: PhantomData,
        }
    }

    fn get_state_count(&self) -> u64 {
        *self.state_count.lock().unwrap()
    }

    fn get_accumulator_count(&self) -> u64 {
        *self.accumulator_count.lock().unwrap()
    }

    fn get_txn_info_count(&self) -> u64 {
        *self.txn_info_count.lock().unwrap()
    }
}

impl SendSyncEventHandler<StateSyncTaskEvent> for TestStateSyncTaskEventHandler {
    fn send_event(&self, event: StateSyncTaskEvent) {
        if event.is_block_accumulator() {
            let mut count: u64 = *self.accumulator_count.lock().unwrap();
            count += 1;
            *self.accumulator_count.lock().unwrap() = count;
        } else if event.is_state() {
            let mut count: u64 = *self.state_count.lock().unwrap();
            count += 1;
            *self.state_count.lock().unwrap() = count;
        } else if event.is_txn_info() {
            let mut count: u64 = *self.txn_info_count.lock().unwrap();
            count += 1;
            *self.txn_info_count.lock().unwrap() = count;
        }
    }
}

#[derive(Clone)]
struct TestTxnInfoEventHandler {
    event: PhantomData<TxnInfoEvent>,
}

impl TestTxnInfoEventHandler {
    fn new() -> Self {
        TestTxnInfoEventHandler { event: PhantomData }
    }
}

impl SendSyncEventHandler<TxnInfoEvent> for TestTxnInfoEventHandler {
    fn send_event(&self, _event: TxnInfoEvent) {}
}

fn gen_block_chain_and_inner(times: u64) -> (Arc<BlockChain>, Inner<DummyNetworkService>) {
    let node_config = Arc::new(NodeConfig::random_for_test());
    let block_chain =
        Arc::new(gen_blockchain_with_blocks_for_test(times, node_config.net()).unwrap());
    let inner = gen_inner(block_chain.clone(), false);
    (block_chain, inner)
}

fn gen_inner(block_chain: Arc<BlockChain>, new_storage: bool) -> Inner<DummyNetworkService> {
    let storage = if new_storage {
        let node_config = Arc::new(NodeConfig::random_for_test());
        gen_blockchain_for_test(node_config.net())
            .unwrap()
            .get_storage()
    } else {
        block_chain.get_storage()
    };
    let pivot = block_chain.current_header();
    let network = DummyNetworkService::new(block_chain.clone());
    Inner::new(
        PeerId::random(),
        (
            pivot.state_root(),
            pivot.parent_block_accumulator_root(),
            pivot.id(),
        ),
        storage,
        network,
    )
}

#[stest::test]
async fn test_state_sync_handle_event() {
    let (block_chain, _) = gen_block_chain_and_inner(1);
    let mut inner = gen_inner(block_chain.clone(), true);
    assert!(!inner.do_finish());

    let header = block_chain.current_header();
    let handler = TestTxnInfoEventHandler::new();
    let peer_id = PeerId::random();

    // state
    let (state_root, is_state_root) = inner.state_sync_task.pop_front().unwrap();
    assert_eq!(state_root, header.state_root());
    let state_node =
        get_state_node_by_node_hash(inner._get_network_client(), peer_id.clone(), state_root)
            .await
            .unwrap();
    let state_task_event = StateSyncTaskEvent::new_state(
        false,
        peer_id.clone(),
        is_state_root,
        state_root,
        Some(state_node),
    );
    inner.handle_state_sync(state_task_event);

    // accumulator
    let accumulator_root = inner.block_accumulator_sync_task.pop_front().unwrap();
    assert_eq!(accumulator_root, header.parent_block_accumulator_root());
    let accumulator_node = get_accumulator_node_by_node_hash(
        inner._get_network_client(),
        peer_id.clone(),
        accumulator_root,
        AccumulatorStoreType::Block,
    )
    .await
    .unwrap();
    let accumulator_task_event = StateSyncTaskEvent::new_accumulator(
        false,
        peer_id.clone(),
        accumulator_root,
        Some(accumulator_node),
    );
    inner.handle_accumulator_sync(accumulator_task_event, Box::new(handler.clone()));

    // txn info
    let block_id = inner.txn_info_sync_task.pop_front().unwrap();
    assert_eq!(block_id, header.id());
    let txn_infos = get_txn_infos(inner._get_network_client(), peer_id.clone(), block_id)
        .await
        .unwrap();
    let txn_info_task_event =
        StateSyncTaskEvent::new_txn_info(false, peer_id.clone(), block_id, txn_infos);
    inner.handle_txn_info_sync(txn_info_task_event, Box::new(handler.clone()));

    assert!(inner.do_finish());
}

#[stest::test]
async fn test_sync_accumulator_node() {
    let (block_chain, inner) = gen_block_chain_and_inner(1);
    let handler = TestStateSyncTaskEventHandler::new();
    let header = block_chain.current_header();
    sync_accumulator_node(
        header.parent_block_accumulator_root(),
        PeerId::random(),
        inner._get_network_client().clone(),
        Box::new(handler.clone()),
    )
    .await;
    assert_eq!(handler.get_accumulator_count(), 1);
}

#[stest::test]
async fn test_sync_state_node() {
    let (block_chain, inner) = gen_block_chain_and_inner(1);
    let handler = TestStateSyncTaskEventHandler::new();
    let header = block_chain.current_header();
    sync_state_node(
        true,
        header.state_root(),
        PeerId::random(),
        inner._get_network_client().clone(),
        Box::new(handler.clone()),
    )
    .await;
    assert_eq!(handler.get_state_count(), 1);
}

#[stest::test]
async fn test_sync_txn_info() {
    let (block_chain, inner) = gen_block_chain_and_inner(1);
    let handler = TestStateSyncTaskEventHandler::new();
    let header = block_chain.current_header();
    sync_txn_info(
        header.id(),
        PeerId::random(),
        inner._get_network_client().clone(),
        Box::new(handler.clone()),
    )
    .await;
    assert_eq!(handler.get_txn_info_count(), 1);
}
