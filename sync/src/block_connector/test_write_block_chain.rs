// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
#![allow(clippy::integer_arithmetic)]
use crate::block_connector::WriteBlockChainService;
use config::NodeConfig;
use consensus::Consensus;
use starcoin_account_api::AccountInfo;
use starcoin_chain::{BlockChain, ChainReader};
use starcoin_chain_service::WriteableChainService;
use starcoin_genesis::Genesis as StarcoinGenesis;
use starcoin_service_registry::bus::BusService;
use starcoin_service_registry::{RegistryAsyncService, RegistryService};
use starcoin_storage::Store;
use starcoin_txpool_mock_service::MockTxPoolService;
use starcoin_types::block::Block;
use starcoin_types::startup_info::StartupInfo;
use starcoin_vm_types::time::TimeService;
use std::sync::Arc;

pub async fn create_writeable_block_chain() -> (
    WriteBlockChainService<MockTxPoolService>,
    Arc<NodeConfig>,
    Arc<dyn Store>,
) {
    let node_config = NodeConfig::random_for_test();
    let node_config = Arc::new(node_config);

    let (storage, chain_info, _) = StarcoinGenesis::init_storage_for_test(node_config.net())
        .expect("init storage by genesis fail.");
    let registry = RegistryService::launch();
    let bus = registry.service_ref::<BusService>().await.unwrap();
    let txpool_service = MockTxPoolService::new();
    (
        WriteBlockChainService::new(
            node_config.clone(),
            StartupInfo::new(chain_info.head().id()),
            storage.clone(),
            txpool_service,
            bus,
            None,
        )
        .unwrap(),
        node_config,
        storage,
    )
}

pub fn gen_blocks(
    times: u64,
    writeable_block_chain_service: &mut WriteBlockChainService<MockTxPoolService>,
    time_service: &dyn TimeService,
) {
    let miner_account = AccountInfo::random();
    if times > 0 {
        for _i in 0..times {
            let block = new_block(
                Some(&miner_account),
                writeable_block_chain_service,
                time_service,
            );
            writeable_block_chain_service.try_connect(block).unwrap();
        }
    }
}

pub fn new_block(
    miner_account: Option<&AccountInfo>,
    writeable_block_chain_service: &mut WriteBlockChainService<MockTxPoolService>,
    time_service: &dyn TimeService,
) -> Block {
    let miner = match miner_account {
        Some(m) => m.clone(),
        None => AccountInfo::random(),
    };
    let miner_address = *miner.address();
    let block_chain = writeable_block_chain_service.get_main();
    let (block_template, _) = block_chain
        .create_block_template(miner_address, None, Vec::new(), vec![], None)
        .unwrap();
    block_chain
        .consensus()
        .create_block(block_template, time_service)
        .unwrap()
}

#[stest::test]
async fn test_block_chain_apply() {
    let times = 10;
    let (mut writeable_block_chain_service, node_config, _) = create_writeable_block_chain().await;
    let net = node_config.net();
    gen_blocks(
        times,
        &mut writeable_block_chain_service,
        net.time_service().as_ref(),
    );
    assert_eq!(
        writeable_block_chain_service
            .get_main()
            .current_header()
            .number(),
        times
    );
}

fn gen_fork_block_chain(
    fork_number: u64,
    node_config: Arc<NodeConfig>,
    times: u64,
    writeable_block_chain_service: &mut WriteBlockChainService<MockTxPoolService>,
) {
    let miner_account = AccountInfo::random();
    if let Some(block_header) = writeable_block_chain_service
        .get_main()
        .get_header_by_number(fork_number)
        .unwrap()
    {
        let mut parent_id = block_header.id();
        let net = node_config.net();
        for _i in 0..times {
            let block_chain = BlockChain::new(
                net.time_service(),
                parent_id,
                writeable_block_chain_service.get_main().get_storage(),
                None,
            )
            .unwrap();
            let (block_template, _) = block_chain
                .create_block_template(*miner_account.address(), None, Vec::new(), vec![], None)
                .unwrap();
            let block = block_chain
                .consensus()
                .create_block(block_template, net.time_service().as_ref())
                .unwrap();
            parent_id = block.id();

            writeable_block_chain_service.try_connect(block).unwrap();
        }
    }
}

#[stest::test]
async fn test_block_chain_forks() {
    let times = 10;
    let (mut writeable_block_chain_service, node_config, _) = create_writeable_block_chain().await;
    let net = node_config.net();
    gen_blocks(
        times,
        &mut writeable_block_chain_service,
        net.time_service().as_ref(),
    );
    assert_eq!(
        writeable_block_chain_service
            .get_main()
            .current_header()
            .number(),
        times
    );

    gen_fork_block_chain(
        0,
        node_config,
        times / 2,
        &mut writeable_block_chain_service,
    );

    assert_eq!(
        writeable_block_chain_service
            .get_main()
            .current_header()
            .number(),
        times
    );
}

#[stest::test]
async fn test_block_chain_switch_main() {
    let times = 10;
    let (mut writeable_block_chain_service, node_config, _) = create_writeable_block_chain().await;
    let net = node_config.net();
    gen_blocks(
        times,
        &mut writeable_block_chain_service,
        net.time_service().as_ref(),
    );
    assert_eq!(
        writeable_block_chain_service
            .get_main()
            .current_header()
            .number(),
        times
    );

    gen_fork_block_chain(
        0,
        node_config,
        2 * times,
        &mut writeable_block_chain_service,
    );

    assert_eq!(
        writeable_block_chain_service
            .get_main()
            .current_header()
            .number(),
        2 * times
    );
}

#[stest::test]
async fn test_block_chain_reset() -> anyhow::Result<()> {
    let times = 10;
    let (mut writeable_block_chain_service, node_config, _) = create_writeable_block_chain().await;
    let net = node_config.net();
    gen_blocks(
        times,
        &mut writeable_block_chain_service,
        net.time_service().as_ref(),
    );
    assert_eq!(
        writeable_block_chain_service
            .get_main()
            .current_header()
            .number(),
        times
    );
    let block = writeable_block_chain_service
        .get_main()
        .get_block_by_number(3)?
        .unwrap();
    writeable_block_chain_service.reset(block.id())?;
    assert_eq!(
        writeable_block_chain_service
            .get_main()
            .current_header()
            .number(),
        3
    );

    assert!(writeable_block_chain_service
        .get_main()
        .get_block_by_number(2)?
        .is_some());
    Ok(())
}
