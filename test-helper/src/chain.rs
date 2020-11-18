// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use starcoin_account_api::AccountInfo;
use starcoin_chain::BlockChain;
use starcoin_consensus::Consensus;
use starcoin_genesis::Genesis;
use starcoin_traits::ChainWriter;
use starcoin_types::genesis_config::ChainNetwork;

pub fn gen_blockchain_for_test(net: &ChainNetwork) -> Result<BlockChain> {
    let (storage, startup_info, _) =
        Genesis::init_storage_for_test(net).expect("init storage by genesis fail.");

    let block_chain = BlockChain::new(net.time_service(), *startup_info.get_main(), storage)?;
    Ok(block_chain)
}

pub fn gen_blockchain_with_blocks_for_test(count: u64, net: &ChainNetwork) -> Result<BlockChain> {
    let mut block_chain = gen_blockchain_for_test(net)?;
    let miner_account = AccountInfo::random();
    for _i in 0..count {
        let (block_template, _) = block_chain
            .create_block_template(
                *miner_account.address(),
                Some(miner_account.public_key.auth_key()),
                None,
                Vec::new(),
                vec![],
                None,
            )
            .unwrap();
        let block = block_chain.consensus().create_block(
            &block_chain,
            block_template,
            net.time_service().as_ref(),
        )?;
        block_chain.apply(block)?;
    }

    Ok(block_chain)
}
