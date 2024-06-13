// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
#![allow(clippy::arithmetic_side_effects)]
use crate::block_connector::test_write_block_chain::create_writeable_dag_block_chain;
use crate::block_connector::WriteBlockChainService;
use anyhow::{bail, Ok};
use starcoin_account_api::AccountInfo;
use starcoin_chain::{BlockChain, ChainReader};
use starcoin_chain_service::WriteableChainService;
use starcoin_config::{genesis_config::G_TEST_DAG_FORK_HEIGHT, ChainNetwork, NodeConfig};
use starcoin_consensus::Consensus;
use starcoin_crypto::HashValue;
use starcoin_txpool_mock_service::MockTxPoolService;
use starcoin_types::block::Block;
use std::sync::Arc;

pub fn gen_dag_blocks(
    times: u64,
    writeable_block_chain_service: &mut WriteBlockChainService<MockTxPoolService>,
    net: &ChainNetwork,
) -> anyhow::Result<HashValue> {
    let miner_account = AccountInfo::random();
    let mut last_block_hash = None;
    if times > 0 {
        for i in 0..times {
            let block = new_dag_block(Some(&miner_account), writeable_block_chain_service, net)?;
            last_block_hash = Some(block.id());
            let e = writeable_block_chain_service.try_connect(block);
            println!("try_connect result: {:?}", e);
            assert!(e.is_ok());
            if (i + 1) % 3 == 0 {
                writeable_block_chain_service.time_sleep(5000000);
            }
        }
        Ok(last_block_hash.ok_or_else(|| anyhow::anyhow!("last block hash is none"))?)
    } else {
        bail!("times must > 0")
    }
}

pub fn new_dag_block(
    miner_account: Option<&AccountInfo>,
    writeable_block_chain_service: &mut WriteBlockChainService<MockTxPoolService>,
    net: &ChainNetwork,
) -> anyhow::Result<Block> {
    let miner = match miner_account {
        Some(m) => m.clone(),
        None => AccountInfo::random(),
    };
    let miner_address = *miner.address();
    let dag_fork_height = writeable_block_chain_service
        .get_main()
        .dag_fork_height()?
        .ok_or_else(|| anyhow::anyhow!("dag fork height is none, can not create dag block"))?;

    if writeable_block_chain_service
        .get_main()
        .current_header()
        .number()
        < dag_fork_height
    {
        let gap = dag_fork_height.saturating_sub(
            writeable_block_chain_service
                .get_main()
                .current_header()
                .number(),
        );
        for _i in 0..gap {
            let block_chain = writeable_block_chain_service.get_main();
            let block_template = block_chain
                .create_block_template(miner_address, None, Vec::new(), vec![], None, None)
                .unwrap()
                .0;
            let block = block_chain
                .consensus()
                .create_block(block_template, net.time_service().as_ref())
                .unwrap();
            writeable_block_chain_service.execute(block.clone())?;
            writeable_block_chain_service.try_connect(block)?;
        }
    }

    let block_chain = writeable_block_chain_service.get_main();
    let (_dag_genesis, tips) = block_chain.current_tips_hash()?;
    let (block_template, _) = block_chain
        .create_block_template(
            miner_address,
            Some(block_chain.current_header().id()),
            Vec::new(),
            vec![],
            None,
            Some(tips),
        )
        .unwrap();
    block_chain
        .consensus()
        .create_block(block_template, net.time_service().as_ref())
}

#[stest::test]
async fn test_dag_block_chain_apply() {
    let times = 12;
    let (mut writeable_block_chain_service, node_config, _) =
        create_writeable_dag_block_chain(G_TEST_DAG_FORK_HEIGHT).await;
    let net = node_config.net();
    let last_header_id = gen_dag_blocks(times, &mut writeable_block_chain_service, net);
    assert_eq!(
        writeable_block_chain_service
            .get_main()
            .current_header()
            .id(),
        last_header_id.unwrap()
    );
}

fn gen_fork_dag_block_chain(
    fork_number: u64,
    node_config: Arc<NodeConfig>,
    times: u64,
    writeable_block_chain_service: &mut WriteBlockChainService<MockTxPoolService>,
) -> Option<HashValue> {
    let miner_account = AccountInfo::random();
    let dag = writeable_block_chain_service.get_dag();
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
                dag.clone(),
            )
            .unwrap();
            let (block_template, _) = block_chain
                .create_block_template(
                    *miner_account.address(),
                    None,
                    Vec::new(),
                    vec![],
                    None,
                    None,
                )
                .unwrap();
            let block = block_chain
                .consensus()
                .create_block(block_template, net.time_service().as_ref())
                .unwrap();
            parent_id = block.id();

            writeable_block_chain_service.try_connect(block).unwrap();
        }
        Some(parent_id)
    } else {
        None
    }
}

#[stest::test(timeout = 120)]
async fn test_block_dag_chain_switch_main() -> anyhow::Result<()> {
    let times = 12;
    let (mut writeable_block_chain_service, node_config, _) =
        create_writeable_dag_block_chain(G_TEST_DAG_FORK_HEIGHT).await;
    let net = node_config.net();
    let mut last_block = gen_dag_blocks(times, &mut writeable_block_chain_service, net)?;
    assert_eq!(
        writeable_block_chain_service
            .get_main()
            .current_header()
            .id(),
        last_block
    );

    last_block = gen_fork_dag_block_chain(
        0,
        node_config,
        3 * times,
        &mut writeable_block_chain_service,
    )
    .ok_or_else(|| anyhow::anyhow!("faile to gen fork dag block chain"))?;

    assert_eq!(
        writeable_block_chain_service
            .get_main()
            .current_header()
            .id(),
        last_block
    );

    Ok(())
}

#[stest::test]
async fn test_block_chain_reset() -> anyhow::Result<()> {
    let times = 10;
    let (mut writeable_block_chain_service, node_config, _) =
        create_writeable_dag_block_chain(G_TEST_DAG_FORK_HEIGHT).await;
    let net = node_config.net();
    let last_block = gen_dag_blocks(times, &mut writeable_block_chain_service, net)?;
    assert_eq!(
        writeable_block_chain_service
            .get_main()
            .current_header()
            .id(),
        last_block
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
