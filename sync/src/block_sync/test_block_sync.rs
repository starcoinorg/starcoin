use crate::block_sync::{BlockIdAndNumber, DataType, Inner, NextTimeEvent, SyncDataEvent};
use crate::sync_event_handle::{CloneSyncEventHandler, SendSyncEventHandler};
use crate::sync_task::SyncTaskState;
use chain::BlockChain;
use config::NodeConfig;
use crypto::HashValue;
use futures_timer::Delay;
use logger::prelude::*;
use starcoin_network_rpc_api::BlockBody;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use test_helper::chain::gen_blockchain_with_blocks_for_test;
use test_helper::DummyNetworkService;
use traits::ChainReader;
use types::block::BlockHeader;

#[derive(Clone)]
struct TestSyncDataEventHandler {
    sync_header_event_count: Arc<Mutex<u64>>,
    sync_body_event_count: Arc<Mutex<u64>>,
    event: PhantomData<SyncDataEvent>,
}

impl TestSyncDataEventHandler {
    fn new() -> Self {
        TestSyncDataEventHandler {
            sync_header_event_count: Arc::new(Mutex::new(0)),
            sync_body_event_count: Arc::new(Mutex::new(0)),
            event: PhantomData,
        }
    }
}

impl SendSyncEventHandler<SyncDataEvent> for TestSyncDataEventHandler {
    fn send_event(&self, event: SyncDataEvent) {
        match event.data_type {
            DataType::Header => {
                let len = event.headers.as_ref().unwrap().len();
                let old_ct: u64 = *self.sync_header_event_count.lock().unwrap();
                let count = old_ct + len as u64;
                *self.sync_header_event_count.lock().unwrap() = count;
                info!(
                    "header_event_count:{}:{}",
                    event
                        .headers
                        .as_ref()
                        .unwrap()
                        .get(len - 1)
                        .unwrap()
                        .number(),
                    count
                );
                assert_eq!(
                    event
                        .headers
                        .as_ref()
                        .unwrap()
                        .get(len - 1)
                        .unwrap()
                        .number(),
                    count
                );
            }
            DataType::Body => {}
        }
    }
}

impl CloneSyncEventHandler<SyncDataEvent> for TestSyncDataEventHandler {
    fn clone_handler(&self) -> Box<dyn SendSyncEventHandler<SyncDataEvent>> {
        Box::new(self.clone())
    }
}

// impl SyncEventHandler<SyncDataEvent> for TestSyncDataEventHandler {}

#[derive(Clone)]
struct TestNextTimeEventHandler {
    next_time_count: Arc<Mutex<u64>>,
    event: PhantomData<NextTimeEvent>,
}

impl TestNextTimeEventHandler {
    fn new() -> Self {
        TestNextTimeEventHandler {
            next_time_count: Arc::new(Mutex::new(0)),
            event: PhantomData,
        }
    }

    fn get_count(&self) -> u64 {
        *self.next_time_count.lock().unwrap()
    }
}

impl SendSyncEventHandler<NextTimeEvent> for TestNextTimeEventHandler {
    fn send_event(&self, _event: NextTimeEvent) {
        let old_ct: u64 = *self.next_time_count.lock().unwrap();
        let count = old_ct + 1;
        *self.next_time_count.lock().unwrap() = count;
    }
}

impl CloneSyncEventHandler<NextTimeEvent> for TestNextTimeEventHandler {
    fn clone_handler(&self) -> Box<dyn SendSyncEventHandler<NextTimeEvent>> {
        Box::new(self.clone())
    }
}

fn gen_block_chain_and_inner(times: u64) -> (Arc<BlockChain>, Inner<DummyNetworkService>) {
    let node_config = Arc::new(NodeConfig::random_for_test());
    let block_chain =
        Arc::new(gen_blockchain_with_blocks_for_test(times, node_config.net()).unwrap());
    let number = 0;
    let block_id = block_chain.find_block_by_number(number).unwrap();
    let id_number = BlockIdAndNumber {
        id: block_id,
        number,
    };

    let network = DummyNetworkService::new(block_chain.clone());
    let inner = Inner::new(0, times, id_number, network, SyncTaskState::Ready);
    (block_chain, inner)
}

fn gen_handlers() -> (TestSyncDataEventHandler, TestNextTimeEventHandler) {
    let handler_1 = TestSyncDataEventHandler::new();
    let handler_2 = TestNextTimeEventHandler::new();
    (handler_1, handler_2)
}

#[stest::test]
async fn test_block_sync_inner() {
    let (handler_1, handler_2) = gen_handlers();
    let times = 15;
    let per = 10;
    let (block_chain, mut inner) = gen_block_chain_and_inner(times);
    assert!(!inner.do_finish());
    let ct = if times / per == 0 {
        times / per
    } else {
        times / per + 1
    };
    for i in 0..ct {
        inner.sync_blocks(Box::new(handler_1.clone()), Box::new(handler_2.clone()));
        Delay::new(Duration::from_millis(1000)).await;
        let next_number = if i != (ct - 1) { (i + 1) * 10 } else { times };

        let next = block_chain
            .get_header_by_number(next_number)
            .unwrap()
            .unwrap();
        inner.update_next(&next);
    }

    let count = handler_2.get_count();
    assert_eq!(count, ct);
    assert!(inner.do_finish());
}

#[stest::test]
async fn test_handle_header() {
    let (_, mut inner) = gen_block_chain_and_inner(1);
    assert!(!inner.do_finish());
    let mut header = BlockHeader::random();
    header.number = 1;
    let mut headers = Vec::new();
    headers.push(header);
    inner.handle_headers(headers);
    assert_eq!(inner._header_size(), 1);
    assert!(!inner.do_finish());
}

#[stest::test]
async fn test_handle_body_empty() {
    let (_, mut inner) = gen_block_chain_and_inner(1);
    assert!(!inner.do_finish());
    let test = BlockIdAndNumber {
        id: HashValue::random(),
        number: 1,
    };
    let mut hashes = Vec::new();
    hashes.push(test);
    inner.handle_bodies(Vec::new(), hashes);
    assert_eq!(inner._body_task_size(), 1);
    assert!(!inner.do_finish());
}

#[stest::test]
async fn test_handle_body() {
    let (_, mut inner) = gen_block_chain_and_inner(1);
    let mut header = BlockHeader::random();
    header.number = 1;

    let mut headers = Vec::new();
    headers.push(header.clone());
    inner.handle_headers(headers);
    assert_eq!(inner._header_size(), 1);
    assert_eq!(inner._body_task_size(), 1);

    let body = BlockBody {
        hash: header.id(),
        transactions: Vec::new(),
        uncles: None,
    };
    let mut bodies = Vec::new();
    bodies.push(body);
    inner.handle_bodies(bodies, Vec::new());
    assert_eq!(inner._header_size(), 0);
    assert_eq!(inner._body_task_size(), 1);
}
