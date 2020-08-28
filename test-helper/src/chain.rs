// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use starcoin_chain::BlockChain;
use starcoin_genesis::Genesis;
use starcoin_types::chain_config::ChainNetwork;

pub fn gen_blockchain_for_test(net: &ChainNetwork) -> Result<BlockChain> {
    let (storage, startup_info, _) =
        Genesis::init_storage_for_test(net).expect("init storage by genesis fail.");

    let block_chain = BlockChain::new(net.consensus(), *startup_info.get_master(), storage, None)?;
    Ok(block_chain)
}
