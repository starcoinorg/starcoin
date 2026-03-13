// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::{
    create_block_template::BlockTemplateResponse, BlockBuilderService, BlockTemplateRequest,
    GenerateBlockEvent, MinerService, UpdateSubscriberNumRequest,
};
use anyhow::Error;
use starcoin_account_service::AccountService;
use starcoin_config::NodeConfig;
use starcoin_genesis::Genesis;
use starcoin_service_registry::mocker::MockHandler;
use starcoin_service_registry::{
    RegistryAsyncService, RegistryService, ServiceContext, ServiceRef,
};
use starcoin_storage::{BlockStore, Storage};
use starcoin_txpool::TxPoolService;
use std::any::Any;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread::sleep as blocking_sleep;
use std::time::Duration;
use tokio::time::{sleep, timeout};

#[derive(Clone)]
struct SlowBlockBuilderMock {
    response: BlockTemplateResponse,
    delay: Duration,
    calls: Arc<AtomicUsize>,
}

impl MockHandler<BlockBuilderService> for SlowBlockBuilderMock {
    fn handle(
        &mut self,
        request: Box<dyn Any>,
        _ctx: &mut ServiceContext<BlockBuilderService>,
    ) -> Box<dyn Any> {
        request
            .downcast::<BlockTemplateRequest>()
            .expect("unexpected block template request");
        self.calls.fetch_add(1, Ordering::SeqCst);
        blocking_sleep(self.delay);
        Box::new(Ok::<BlockTemplateResponse, Error>(self.response.clone()))
    }
}

async fn prepare_template_response() -> (Arc<NodeConfig>, Arc<Storage>, BlockTemplateResponse) {
    let mut config = NodeConfig::random_for_test();
    config.miner.disable_miner_client = Some(true);
    config.miner.disable_mint_empty_block = Some(false);
    let node_config = Arc::new(config);

    let (storage, _chain_info, genesis) =
        Genesis::init_storage_for_test(node_config.net()).unwrap();
    let chain_header = storage
        .get_block_header_by_hash(genesis.block().id())
        .unwrap()
        .unwrap();
    let txpool = TxPoolService::new(node_config.clone(), storage.clone(), chain_header, None);

    let registry = RegistryService::launch();
    registry.put_shared(node_config.clone()).await.unwrap();
    registry.put_shared(storage.clone()).await.unwrap();
    registry.put_shared(txpool).await.unwrap();
    registry
        .register_mocker(AccountService::mock().unwrap())
        .await
        .unwrap();

    let block_builder = registry.register::<BlockBuilderService>().await.unwrap();
    let response = block_builder
        .send(BlockTemplateRequest)
        .await
        .unwrap()
        .unwrap();
    registry.shutdown_system().await.unwrap();

    (node_config, storage, response)
}

async fn register_miner_with_mock_builder(
    node_config: Arc<NodeConfig>,
    storage: Arc<Storage>,
    mocker: SlowBlockBuilderMock,
) -> (ServiceRef<MinerService>, ServiceRef<RegistryService>) {
    let startup_info = storage.get_startup_info().unwrap().unwrap();
    let chain_header = storage
        .get_block_header_by_hash(startup_info.main)
        .unwrap()
        .unwrap();
    let txpool = TxPoolService::new(node_config.clone(), storage.clone(), chain_header, None);

    let registry = RegistryService::launch();
    registry.put_shared(node_config).await.unwrap();
    registry.put_shared(storage).await.unwrap();
    registry.put_shared(txpool).await.unwrap();
    registry
        .register_mocker(AccountService::mock().unwrap())
        .await
        .unwrap();
    registry.register_mocker(mocker).await.unwrap();

    let miner = registry.register::<MinerService>().await.unwrap();
    (miner, registry)
}

#[stest::test]
async fn test_generate_block_event_keeps_miner_service_responsive() {
    let (node_config, storage, response) = prepare_template_response().await;
    let calls = Arc::new(AtomicUsize::new(0));
    let mocker = SlowBlockBuilderMock {
        response,
        delay: Duration::from_secs(1),
        calls: calls.clone(),
    };
    let (miner, registry) = register_miner_with_mock_builder(node_config, storage, mocker).await;

    miner
        .send(UpdateSubscriberNumRequest { number: Some(1) })
        .await
        .unwrap();
    sleep(Duration::from_millis(50)).await;

    for _ in 0..64 {
        miner.notify(GenerateBlockEvent::new_break(true)).unwrap();
    }

    timeout(
        Duration::from_millis(200),
        miner.send(UpdateSubscriberNumRequest { number: None }),
    )
    .await
    .expect("miner service should stay responsive while building")
    .unwrap();

    sleep(Duration::from_millis(200)).await;
    assert_eq!(
        calls.load(Ordering::SeqCst),
        1,
        "only one block template build should be inflight"
    );

    sleep(Duration::from_millis(1200)).await;
    assert!(
        calls.load(Ordering::SeqCst) <= 2,
        "refresh events should coalesce into a single pending rebuild"
    );

    registry.shutdown_system().await.unwrap();
}

#[stest::test]
async fn test_subscriber_registration_triggers_first_job_build() {
    let (node_config, storage, response) = prepare_template_response().await;
    let calls = Arc::new(AtomicUsize::new(0));
    let mocker = SlowBlockBuilderMock {
        response,
        delay: Duration::from_millis(50),
        calls: calls.clone(),
    };
    let (miner, registry) = register_miner_with_mock_builder(node_config, storage, mocker).await;

    let initial = miner
        .send(UpdateSubscriberNumRequest { number: Some(1) })
        .await
        .unwrap();
    assert!(initial.is_none());

    let minted = timeout(Duration::from_secs(2), async {
        loop {
            if let Some(event) = miner
                .send(UpdateSubscriberNumRequest { number: None })
                .await
                .unwrap()
            {
                break event;
            }
            sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .expect("first subscriber should trigger an initial mint job");

    assert!(!minted.minting_blob.is_empty());
    assert_eq!(calls.load(Ordering::SeqCst), 1);

    registry.shutdown_system().await.unwrap();
}

#[stest::test]
async fn test_slow_block_template_build_still_dispatches_job() {
    let (node_config, storage, response) = prepare_template_response().await;
    let calls = Arc::new(AtomicUsize::new(0));
    let mocker = SlowBlockBuilderMock {
        response,
        delay: Duration::from_secs(6),
        calls: calls.clone(),
    };
    let (miner, registry) = register_miner_with_mock_builder(node_config, storage, mocker).await;

    let initial = miner
        .send(UpdateSubscriberNumRequest { number: Some(1) })
        .await
        .unwrap();
    assert!(initial.is_none());

    timeout(
        Duration::from_millis(200),
        miner.send(UpdateSubscriberNumRequest { number: None }),
    )
    .await
    .expect("miner service should stay responsive while waiting for a slow template build")
    .unwrap();

    let minted = timeout(Duration::from_secs(8), async {
        loop {
            if let Some(event) = miner
                .send(UpdateSubscriberNumRequest { number: None })
                .await
                .unwrap()
            {
                break event;
            }
            sleep(Duration::from_millis(100)).await;
        }
    })
    .await
    .expect("slow block template builds should still publish a mint job");

    assert!(!minted.minting_blob.is_empty());
    assert_eq!(calls.load(Ordering::SeqCst), 1);

    registry.shutdown_system().await.unwrap();
}
