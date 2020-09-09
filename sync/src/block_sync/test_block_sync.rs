use crate::block_sync::{
    BlockIdAndNumber, CloneEventHandler, DataType, EventHandler, Inner, SendEventHandler,
    SyncDataEvent,
};
use crate::sync_task::SyncTaskState;
use config::NodeConfig;
use futures_timer::Delay;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use test_helper::chain::gen_blockchain_with_blocks_for_test;
use test_helper::DummyNetworkService;
use traits::ChainReader;

#[derive(Clone)]
struct TestEventHandler {
    sync_header_event_count: Arc<Mutex<u64>>,
    sync_body_event_count: Arc<Mutex<u64>>,
    next_time_count: Arc<Mutex<u64>>,
}

impl TestEventHandler {
    fn new() -> Self {
        TestEventHandler {
            sync_header_event_count: Arc::new(Mutex::new(0)),
            sync_body_event_count: Arc::new(Mutex::new(0)),
            next_time_count: Arc::new(Mutex::new(0)),
        }
    }
}

impl SendEventHandler for TestEventHandler {
    fn send_event(&self, event: SyncDataEvent) {
        match event.data_type {
            DataType::Header => {
                let len = event.headers.len();
                let old_ct: u64 = *self.sync_header_event_count.lock().unwrap();
                let count = old_ct + len as u64;
                *self.sync_header_event_count.lock().unwrap() = count;
                assert_eq!(event.headers.get(len - 1).unwrap().number(), count);
            }
            DataType::Body => {}
        }
    }

    fn next_time(&self) {
        let old_ct: u64 = *self.next_time_count.lock().unwrap();
        let count = old_ct + 1;
        *self.next_time_count.lock().unwrap() = count;
        let header_count: u64 = *self.sync_header_event_count.lock().unwrap();
        let ct = if header_count % 10 == 0 {
            header_count / 10
        } else {
            header_count / 10 + 1
        };

        assert_eq!(ct, count);
    }
}

impl CloneEventHandler for TestEventHandler {
    fn clone_handler(&self) -> Box<dyn SendEventHandler> {
        Box::new(self.clone())
    }
}

impl EventHandler for TestEventHandler {}

#[stest::test]
async fn test_block_sync_inner() {
    let handler = TestEventHandler::new();
    let node_config = Arc::new(NodeConfig::random_for_test());
    let times = 15;
    let block_chain =
        Arc::new(gen_blockchain_with_blocks_for_test(times, node_config.net()).unwrap());
    let number = 0;
    let block_id = block_chain.find_block_by_number(number).unwrap();
    let id_number = BlockIdAndNumber {
        id: block_id,
        number,
    };

    let network = DummyNetworkService::new(block_chain.clone());
    let mut inner = Inner::new(0, times, id_number, network, SyncTaskState::Ready);
    assert!(!inner.do_finish());
    let ct = if times / 10 == 0 {
        times / 10
    } else {
        times / 10 + 1
    };
    for i in 0..ct {
        inner.sync_blocks(Box::new(handler.clone()));
        Delay::new(Duration::from_millis(1000)).await;
        if i != (ct - 1) {
            let next = block_chain
                .get_header_by_number((i + 1) * 10)
                .unwrap()
                .unwrap();
            inner.update_next(&next);
        }
    }
    // assert!(inner.do_finish());
}
