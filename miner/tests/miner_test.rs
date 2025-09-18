// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use async_std::future::timeout;
use async_std::stream::StreamExt;
use starcoin_account_service::AccountService;
use starcoin_config::NodeConfig;
use starcoin_consensus::Consensus;
use starcoin_dag::service::pruning_point_service::PruningPointService;
use starcoin_genesis::Genesis;
use starcoin_logger::prelude::info;
use starcoin_miner::{
    BlockBuilderService, BlockHeaderExtra, BlockTemplateRequest, BlockTemplateResponse,
    MinerService, NewHeaderChannel, NewHeaderService, SubmitSealRequest,
};
use starcoin_service_registry::{
    ActorService, EventHandler, RegistryAsyncService, RegistryService, ServiceFactory,
};
use starcoin_storage::BlockStore;
use starcoin_sync::block_connector::BlockConnectorService;
use starcoin_txpool::TxPoolService;
use starcoin_types::{system_events::GenerateBlockEvent, U256};
use std::sync::Arc;
use std::time::Duration;

struct TestMinerService {
    pub wait_result_sender: Option<futures::channel::mpsc::UnboundedSender<()>>,
}

impl TestMinerService {
    pub fn new() -> Self {
        Self {
            wait_result_sender: None,
        }
    }
}

impl ServiceFactory<Self> for TestMinerService {
    fn create(_ctx: &mut starcoin_service_registry::ServiceContext<Self>) -> anyhow::Result<Self> {
        Ok(Self::new())
    }
}

impl ActorService for TestMinerService {
    fn service_name() -> &'static str {
        std::any::type_name::<Self>()
    }

    fn started(
        &mut self,
        ctx: &mut starcoin_service_registry::ServiceContext<Self>,
    ) -> anyhow::Result<()> {
        ctx.subscribe::<BlockTemplateResponse>();
        let (sender, mut receiver) = futures::channel::mpsc::unbounded::<()>();
        self.wait_result_sender = Some(sender);

        ctx.run_later(
            Duration::from_secs(20),
            move |_ctx: &mut starcoin_service_registry::ServiceContext<'_, Self>| {
                async_std::task::block_on(async {
                    match timeout(Duration::from_secs(1), receiver.next()).await {
                        Ok(_) => info!("receive the block template response"),
                        Err(_) => panic!("not receive the block template response"),
                    }
                });
            },
        );
        Ok(())
    }

    fn stopped(
        &mut self,
        ctx: &mut starcoin_service_registry::ServiceContext<Self>,
    ) -> anyhow::Result<()> {
        ctx.unsubscribe::<BlockTemplateResponse>();

        info!("stoped receive the block template response and stop the testing service");
        Ok(())
    }
}

impl EventHandler<Self, BlockTemplateResponse> for TestMinerService {
    fn handle_event(
        &mut self,
        msg: BlockTemplateResponse,
        ctx: &mut starcoin_service_registry::ServiceContext<Self>,
    ) {
        let response = msg.template;
        assert_eq!(response.number, 1);

        let miner = ctx.service_ref::<MinerService>().unwrap().clone();
        miner.notify(GenerateBlockEvent::new_break(false)).unwrap();

        std::thread::sleep(Duration::from_millis(200));
        miner.notify(GenerateBlockEvent::new_break(true)).unwrap();
        std::thread::sleep(Duration::from_millis(200));
        // Generate a event
        let diff = U256::from(1024);
        let minting_blob = vec![0u8; 76];

        let config = ctx.get_shared::<Arc<NodeConfig>>().unwrap();
        let nonce = config
            .net()
            .genesis_config()
            .consensus()
            .solve_consensus_nonce(&minting_blob, diff, config.net().time_service().as_ref());
        miner
            .try_send(SubmitSealRequest::new(
                minting_blob,
                nonce,
                BlockHeaderExtra::new([0u8; 4]),
            ))
            .unwrap();

        if let Some(sender) = self.wait_result_sender.as_mut() {
            sender.start_send(()).unwrap();
        }
        info!("notify testing service to stop");
    }
}

#[stest::test]
async fn test_miner_service() {
    let mut config = NodeConfig::random_for_dag_test();
    config.miner.disable_mint_empty_block = Some(false);
    let registry = RegistryService::launch();
    let node_config = Arc::new(config.clone());
    registry.put_shared(node_config.clone()).await.unwrap();
    let (storage, _chain_info, genesis, dag) =
        Genesis::init_storage_for_test(config.net()).unwrap();
    registry.put_shared(storage.clone()).await.unwrap();
    registry.put_shared(dag).await.unwrap();

    let genesis_hash = genesis.block().id();
    registry.put_shared(genesis).await.unwrap();
    let chain_header = storage
        .get_block_header_by_hash(genesis_hash)
        .unwrap()
        .unwrap();

    let txpool = TxPoolService::new(node_config.clone(), storage.clone(), chain_header, None);
    registry.put_shared(txpool).await.unwrap();
    registry
        .register_mocker(AccountService::mock().unwrap())
        .await
        .unwrap();

    registry.register::<PruningPointService>().await.unwrap();

    registry
        .register::<BlockConnectorService<TxPoolService>>()
        .await
        .unwrap();

    registry.put_shared(NewHeaderChannel::new()).await.unwrap();
    registry.register::<NewHeaderService>().await.unwrap();

    let miner = registry.register::<MinerService>().await;
    assert!(miner.is_ok());

    let template = registry.register::<BlockBuilderService>().await.unwrap();
    registry.register::<TestMinerService>().await.unwrap();

    template
        .notify(BlockTemplateRequest)
        .expect("failed to send template request");

    std::thread::sleep(Duration::from_secs(30));

    registry
        .shutdown_system()
        .await
        .expect("failed to stop registry service");
}
