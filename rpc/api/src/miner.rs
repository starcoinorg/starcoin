// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

pub use self::gen_client::Client as MinerClient;
use crate::types::MintedBlockView;
use crate::FutureResult;
use openrpc_derive::openrpc;
use starcoin_types::blockhash::BlockLevel;
use starcoin_types::system_events::MintBlockEvent;

#[openrpc]
pub trait MinerApi {
    /// submit mining seal
    #[rpc(name = "mining.submit")]
    fn submit(
        &self,
        minting_blob: String,
        nonce: u32,
        extra: String,
        block_level: BlockLevel,
    ) -> FutureResult<MintedBlockView>;
    /// get current mining job
    #[rpc(name = "mining.get_job")]
    fn get_job(&self) -> FutureResult<Option<MintBlockEvent>>;
}

#[test]
fn test() {
    let schema = self::gen_schema();
    let j = serde_json::to_string_pretty(&schema).unwrap();
    println!("{}", j);
}
