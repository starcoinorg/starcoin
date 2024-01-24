// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
#![allow(clippy::integer_arithmetic)]
use crate::block_connector::test_write_block_chain::create_writeable_block_chain;
use crate::block_connector::WriteBlockChainService;
use starcoin_account_api::AccountInfo;
use starcoin_chain::{BlockChain, ChainReader};
use starcoin_chain_service::WriteableChainService;
use starcoin_config::NodeConfig;
use starcoin_consensus::Consensus;
use starcoin_crypto::HashValue;
use starcoin_time_service::TimeService;
use starcoin_txpool_mock_service::MockTxPoolService;
use starcoin_types::block::Block;
use std::sync::Arc;

pub fn gen_dag_blocks(
    times: u64,
    writeable_block_chain_service: &mut WriteBlockChainService<MockTxPoolService>,
    time_service: &dyn TimeService,
) -> Option<HashValue> {
    let miner_account = AccountInfo::random();
    let mut last_block_hash = None;
    if times > 0 {
        for i in 0..times {
            let block = new_dag_block(
                Some(&miner_account),
                writeable_block_chain_service,
                time_service,
            );
            last_block_hash = Some(block.id());
            let e = writeable_block_chain_service.try_connect(block);
            println!("try_connect result: {:?}", e);
            assert!(e.is_ok());
            if (i + 1) % 3 == 0 {
                writeable_block_chain_service.time_sleep(5000000);
            }
        }
        last_block_hash
    } else {
        None
    }

    // match result {
    //     super::write_block_chain::ConnectOk::Duplicate(block)
    //     | super::write_block_chain::ConnectOk::ExeConnectMain(block)
    //     | super::write_block_chain::ConnectOk::ExeConnectBranch(block)
    //     | super::write_block_chain::ConnectOk::Connect(block) => Some(block.header().id()),
    //     super::write_block_chain::ConnectOk::DagConnected
    //     | super::write_block_chain::ConnectOk::MainDuplicate
    //     | super::write_block_chain::ConnectOk::DagPending
    //     | super::write_block_chain::ConnectOk::DagConnectMissingBlock => {
    //         unreachable!("should not reach here, result: {:?}", result);
    //     }
    // }
}

pub fn new_dag_block(
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
    let tips = block_chain.current_tips_hash().expect("failed to get tips");
    let (block_template, _) = block_chain
        .create_block_template(miner_address, None, Vec::new(), vec![], None, tips)
        .unwrap();
    block_chain
        .consensus()
        .create_block(block_template, time_service)
        .unwrap()
}

#[stest::test]
async fn test_dag_block_chain_apply() {
    let times = 12;
    let (mut writeable_block_chain_service, node_config, _) = create_writeable_block_chain().await;
    let net = node_config.net();
    let last_header_id = gen_dag_blocks(
        times,
        &mut writeable_block_chain_service,
        net.time_service().as_ref(),
    );
    assert_eq!(
        writeable_block_chain_service
            .get_main()
            .current_header()
            .id(),
        last_header_id.unwrap()
    );
    println!("finish test_block_chain_apply");
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
async fn test_block_chain_switch_main() {
    let times = 12;
    let (mut writeable_block_chain_service, node_config, _) = create_writeable_block_chain().await;
    let net = node_config.net();
    let mut last_block = gen_dag_blocks(
        times,
        &mut writeable_block_chain_service,
        net.time_service().as_ref(),
    );
    assert_eq!(
        writeable_block_chain_service
            .get_main()
            .current_header()
            .id(),
        last_block.unwrap()
    );

    last_block = gen_fork_dag_block_chain(
        0,
        node_config,
        2 * times,
        &mut writeable_block_chain_service,
    );

    assert_eq!(
        writeable_block_chain_service
            .get_main()
            .current_header()
            .id(),
        last_block.unwrap()
    );
}

#[stest::test]
async fn test_block_chain_reset() -> anyhow::Result<()> {
    let times = 10;
    let (mut writeable_block_chain_service, node_config, _) = create_writeable_block_chain().await;
    let net = node_config.net();
    let last_block = gen_dag_blocks(
        times,
        &mut writeable_block_chain_service,
        net.time_service().as_ref(),
    );
    assert_eq!(
        writeable_block_chain_service
            .get_main()
            .current_header()
            .id(),
        last_block.unwrap()
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
