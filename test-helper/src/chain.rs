// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::dao::{
    execute_script_on_chain_config, modify_on_chain_config_by_dao_block, on_chain_config_type_tag,
    vote_flexi_dag_config,
};
use anyhow::{anyhow, Result};
use starcoin_account_api::AccountInfo;
use starcoin_chain::ChainWriter;
use starcoin_chain::{BlockChain, ChainReader};
use starcoin_config::ChainNetwork;
use starcoin_consensus::Consensus;
use starcoin_genesis::Genesis;
use starcoin_types::account::Account;
use starcoin_types::block::{BlockNumber, TEST_FLEXIDAG_FORK_HEIGHT_NEVER_REACH};
use starcoin_vm_types::on_chain_config::FlexiDagConfig;

pub fn gen_blockchain_for_test(net: &ChainNetwork) -> Result<BlockChain> {
    let (storage, chain_info, _, dag) =
        Genesis::init_storage_for_test(net)
            .expect("init storage by genesis fail.");

    let block_chain = BlockChain::new(
        net.time_service(),
        chain_info.head().id(),
        storage,
        None,
        dag,
    )?;
    Ok(block_chain)
}

pub fn gen_blockchain_for_dag_test(
    net: &ChainNetwork,
    fork_number: BlockNumber,
) -> Result<BlockChain> {
    let (storage, chain_info, _, dag) =
        Genesis::init_storage_for_test(net).expect("init storage by genesis fail.");

    let block_chain = BlockChain::new(
        net.time_service(),
        chain_info.head().id(),
        storage,
        None,
        dag,
    )?;

    let alice = Account::new();
    let block_chain = modify_on_chain_config_by_dao_block(
        alice,
        block_chain,
        net,
        vote_flexi_dag_config(net, fork_number),
        on_chain_config_type_tag(FlexiDagConfig::type_tag()),
        execute_script_on_chain_config(net, FlexiDagConfig::type_tag(), 0u64),
    )?;

    if block_chain.current_header().number() >= fork_number {
        return Err(anyhow!("invalid fork_number"));
    }

    Ok(block_chain)
}

pub fn gen_blockchain_with_blocks_for_test(count: u64, net: &ChainNetwork) -> Result<BlockChain> {
    let mut block_chain = gen_blockchain_for_test(net)?;
    let miner_account = AccountInfo::random();
    for _i in 0..count {
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
            .create_block(block_template, net.time_service().as_ref())?;
        block_chain.apply(block)?;
    }

    Ok(block_chain)
}
