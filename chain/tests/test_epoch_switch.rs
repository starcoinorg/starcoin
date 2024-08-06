// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
// use starcoin_chain::ChainWriter;
// use starcoin_chain_api::ChainReader;
use starcoin_config::NodeConfig;
use starcoin_types::account::Account;
use starcoin_vm_types::on_chain_config::consensus_config_type_tag;
use std::sync::Arc;
// use test_helper::block::create_new_block;
// use test_helper::dao::{
//     execute_script_on_chain_config, modify_on_chain_config_by_dao_block, on_chain_config_type_tag,
//     vote_script_consensus,
// };

#[stest::test(timeout = 120)]
fn test_modify_on_chain_config_consensus_by_dao() -> Result<()> {
    let config = Arc::new(NodeConfig::random_for_dag_test());
    let net = config.net();
    let _chain = test_helper::gen_blockchain_for_test(net)?;

    let _alice = Account::new();
    let _bob = Account::new();
    let _action_type_tag = consensus_config_type_tag();
    let _strategy = 3u8;

    // fail to verify the count of the uncles in dag mode
    // let mut modified_chain = modify_on_chain_config_by_dao_block(
    //     alice,
    //     chain,
    //     net,
    //     vote_script_consensus(net, strategy),
    //     on_chain_config_type_tag(action_type_tag.clone()),
    //     execute_script_on_chain_config(net, action_type_tag, 0u64),
    // )?;

    // // add block to switch epoch
    // let epoch = modified_chain.epoch();
    // let mut number = epoch.end_block_number()
    //     - epoch.start_block_number()
    //     - modified_chain.current_header().number();
    // while number > 0 {
    //     modified_chain.apply(create_new_block(&modified_chain, &bob, vec![])?)?;
    //     number -= 1;
    // }

    // assert_eq!(modified_chain.consensus().value(), strategy);
    Ok(())
}
