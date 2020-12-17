// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use crate::module::map_err;
use jsonrpc_core::Result;
use starcoin_miner::MinerService;
use starcoin_rpc_api::miner::MinerApi;
use starcoin_service_registry::ServiceRef;
use starcoin_types::system_events::SubmitSealEvent;

pub struct MinerRpcImpl {
    miner_service: ServiceRef<MinerService>,
}

impl MinerRpcImpl {
    pub fn new(miner_service: ServiceRef<MinerService>) -> Self {
        Self { miner_service }
    }
}

impl MinerApi for MinerRpcImpl {
    fn submit(&self, minting_blob: Vec<u8>, nonce: u32) -> Result<()> {
        self.miner_service
            .notify(SubmitSealEvent {
                nonce,
                minting_blob,
            })
            .map_err(|e| map_err(e.into()))
    }
}
