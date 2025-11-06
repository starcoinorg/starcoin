// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, Result};
use starcoin_account_service::AccountService;
use starcoin_chain::verifier::VerifyWithoutConsensus;
use starcoin_chain::{BlockChain, ChainReader};
use starcoin_config::{NodeConfig, TimeService};
use starcoin_consensus::Consensus;
use starcoin_crypto::HashValue;
use starcoin_dag::blockdag::BlockDAG;
use starcoin_dag::service::pruning_point_service::PruningPointService;
use starcoin_genesis::Genesis;
use starcoin_logger::prelude::info;
use starcoin_miner::create_block_template::block_builder_service::{BlockTemplateCallBack, Inner};
use starcoin_miner::{
    BlockBuilderService, BlockHeaderExtra, BlockTemplateRequest, MinerService, MintBlockEvent,
    NewHeaderChannel, NewHeaderService, SubmitSealRequest,
};
use starcoin_service_registry::{
    ActorService, EventHandler, RegistryAsyncService, RegistryService, ServiceFactory,
};
use starcoin_storage::{BlockStore, Store, Store2};
use starcoin_sync::block_connector::BlockConnectorService;
use starcoin_transaction_builder::vm2::build_transfer_from_association;
use starcoin_txpool::TxPoolService;
use starcoin_txpool_api::TxPoolSyncService;
use starcoin_txpool_mock_service::MockTxPoolService;
use starcoin_types::block::{BlockHeader, BlockTemplate};
use starcoin_types::{system_events::GenerateBlockEvent, U256};
use starcoin_vm2_account_api::AccountInfo;
use starcoin_vm2_crypto::keygen::KeyGen;
use starcoin_vm2_types::account::DEFAULT_EXPIRATION_TIME;
use starcoin_vm2_types::transaction::SignedUserTransaction;
use starcoin_vm2_types::{account_address, account_config};
use starcoin_vm2_vm_types::state_view::StateReaderExt;
use std::sync::mpsc::Sender;
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
        ctx.subscribe::<MintBlockEvent>();
        let (sender, mut receiver) = futures::channel::mpsc::unbounded::<()>();
        self.wait_result_sender = Some(sender);

        ctx.run_later(
            Duration::from_secs(20),
            move |_ctx: &mut starcoin_service_registry::ServiceContext<'_, Self>| match receiver
                .try_next()
            {
                Ok(_) => (),
                Err(e) => panic!("Failed to receive result: {}", e),
            },
        );
        Ok(())
    }

    fn stopped(
        &mut self,
        ctx: &mut starcoin_service_registry::ServiceContext<Self>,
    ) -> anyhow::Result<()> {
        ctx.unsubscribe::<MintBlockEvent>();

        info!("stoped receive the block template response and stop the testing service");
        Ok(())
    }
}

impl EventHandler<Self, MintBlockEvent> for TestMinerService {
    fn handle_event(
        &mut self,
        msg: MintBlockEvent,
        ctx: &mut starcoin_service_registry::ServiceContext<Self>,
    ) {
        let response = msg.block_number;
        assert_eq!(response, 1);

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
            .genesis_config2()
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
    let (storage, storage2, _chain_info, genesis, dag) =
        Genesis::init_storage_for_test(config.net()).unwrap();
    registry.put_shared(storage.clone()).await.unwrap();
    registry.put_shared(dag).await.unwrap();

    let genesis_hash = genesis.block().id();
    registry.put_shared(genesis).await.unwrap();
    let chain_header = storage
        .get_block_header_by_hash(genesis_hash)
        .unwrap()
        .unwrap();

    let txpool = TxPoolService::new(
        node_config.clone(),
        storage.clone(),
        storage2,
        chain_header,
        None,
    );
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
        .notify(BlockTemplateRequest {
            event: GenerateBlockEvent::default(),
        })
        .expect("failed to send template request");

    std::thread::sleep(Duration::from_secs(30));

    registry
        .shutdown_system()
        .await
        .expect("failed to stop registry service");
}

struct TestTemplateNotify {
    chain: BlockChain,
    finish_sender: Sender<Result<()>>,
    check_transaction_id: Vec<HashValue>,
}

impl TestTemplateNotify {
    pub fn new(
        finish_sender: Sender<Result<()>>,
        check_transaction_id: Vec<HashValue>,
        time_service: Arc<dyn TimeService>,
        head_block_hash: HashValue,
        storage: Arc<dyn Store>,
        storage2: Arc<dyn Store2>,
        dag: BlockDAG,
    ) -> Result<Self> {
        let chain = BlockChain::new(time_service, head_block_hash, storage, storage2, None, dag)?;
        Ok(Self {
            chain,
            finish_sender,
            check_transaction_id,
        })
    }
}

impl BlockTemplateCallBack for TestTemplateNotify {
    fn block_template_callback(
        &mut self,
        _parent: BlockHeader,
        block_template: BlockTemplate,
    ) -> Result<()> {
        let block =
            block_template.into_block(0, starcoin_types::block::BlockHeaderExtra::new([0u8; 4]));
        let result = self
            .chain
            .apply_with_verifier::<VerifyWithoutConsensus>(block);
        self.finish_sender.send(match result {
            Ok(executed_block) => {
                let transaction_ids = executed_block
                    .block()
                    .body
                    .transactions2
                    .iter()
                    .map(|t| t.id())
                    .collect::<Vec<_>>();
                println!(
                    "jacktest: transaction ids in executed block: {:?}",
                    transaction_ids
                );
                assert!(transaction_ids.contains(
                    self.check_transaction_id
                        .first()
                        .expect("3 transactions lack index 0")
                ));
                assert!(!transaction_ids.contains(
                    self.check_transaction_id
                        .get(1)
                        .expect("3 transactions lack index 2")
                ));
                assert!(!transaction_ids.contains(
                    self.check_transaction_id
                        .get(2)
                        .expect("3 transactions lack index 2")
                ));
                Ok(())
            }
            Err(e) => Err(e),
        })?;
        Ok(())
    }
}

#[stest::test]
pub fn test_open_block_and_execute() -> Result<()> {
    let config = Arc::new(NodeConfig::random_for_test());
    let chain = test_helper::gen_blockchain_for_test(config.net())?;

    let account_reader = chain.chain_state_reader2();
    let association_sequence_num =
        account_reader.get_sequence_number(account_config::association_address())?;
    let (_receive_prikey, receive_public_key) = KeyGen::from_os_rng().generate_keypair();
    let receiver = account_address::from_public_key(&receive_public_key);
    let txn1: SignedUserTransaction = build_transfer_from_association(
        receiver,
        association_sequence_num,
        50_000_000,
        config.net().time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
        config.net().chain_id().id().into(),
        config.net().genesis_config2(),
    )
    .try_into()?;

    let txn2: SignedUserTransaction = build_transfer_from_association(
        receiver,
        association_sequence_num.saturating_add(3), // will fail for sequence number too new
        50_000_000,
        config.net().time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
        config.net().chain_id().id().into(),
        config.net().genesis_config2(),
    )
    .try_into()?;

    let txn3: SignedUserTransaction = build_transfer_from_association(
        receiver,
        association_sequence_num, // will fail for sequence number too old
        30_000_000,
        config.net().time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
        config.net().chain_id().id().into(),
        config.net().genesis_config2(),
    )
    .try_into()?;
    let check_transaction_ids = vec![txn1.clone(), txn2.clone(), txn3.clone()]
        .into_iter()
        .map(|t| t.id())
        .collect::<Vec<_>>();

    let txpool = MockTxPoolService::new();
    txpool.add_txns_multi_signed(vec![txn1.into(), txn2.into(), txn3.into()], false, None)?;

    let miner_account_info = AccountInfo::random();

    let mut create_block_template_service = Inner::new(
        chain.current_header(),
        chain.get_storage(),
        chain.get_storage2(),
        txpool,
        config.miner.block_gas_limit,
        miner_account_info,
        chain.dag(),
        config.clone(),
        None,
        None,
    )?;

    let (sender, receiver) = std::sync::mpsc::channel::<Result<()>>();
    let callback = TestTemplateNotify::new(
        sender,
        check_transaction_ids,
        config.net().time_service(),
        chain.current_header().id(),
        chain.get_storage(),
        chain.get_storage2(),
        chain.dag(),
    )?;

    create_block_template_service.create_block_template(1, Box::new(callback))?;

    if let Err(e) = receiver.recv_timeout(std::time::Duration::from_secs(10))? {
        bail!("failed to create and execute a block for: {:?}", e);
    }

    Ok(())
}
