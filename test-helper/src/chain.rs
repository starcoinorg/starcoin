// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use starcoin_account_api::AccountInfo;
use starcoin_chain::BlockChain;
use starcoin_chain::ChainWriter;
use starcoin_config::ChainNetwork;
use starcoin_consensus::Consensus;
use starcoin_genesis::Genesis;

pub fn gen_blockchain_for_test(net: &ChainNetwork) -> Result<BlockChain> {
    let (storage, chain_info, _) =
        Genesis::init_storage_for_test(net).expect("init storage by genesis fail.");

    let block_chain = BlockChain::new(net.time_service(), chain_info.head().id(), storage, None)?;
    Ok(block_chain)
}

pub fn gen_blockchain_with_blocks_for_test(count: u64, net: &ChainNetwork) -> Result<BlockChain> {
    let mut block_chain = gen_blockchain_for_test(net)?;
    let miner_account = AccountInfo::random();
    for _i in 0..count {
        let (block_template, _) = block_chain
            .create_block_template(*miner_account.address(), None, Vec::new(), vec![], None)
            .unwrap();
        let block = block_chain
            .consensus()
            .create_block(block_template, net.time_service().as_ref())?;
        block_chain.apply(block)?;
    }

    Ok(block_chain)
}
