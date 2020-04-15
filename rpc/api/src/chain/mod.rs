// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use crate::FutureResult;
use jsonrpc_derive::rpc;
use starcoin_crypto::HashValue;
use starcoin_types::block::Block;
use starcoin_types::startup_info::ChainInfo;

#[rpc]
pub trait ChainApi {
    // Get chain head info
    #[rpc(name = "chain.head")]
    fn head(&self) -> FutureResult<ChainInfo>;
    // Get chain block info
    #[rpc(name = "chain.get_block_by_hash")]
    fn get_block_by_hash(&self, hash: HashValue) -> FutureResult<Block>;
}
