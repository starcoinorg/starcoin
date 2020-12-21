// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use crate::module::{map_err, RpcError};
use jsonrpc_core::{ErrorCode, Result};
use starcoin_miner::MinerService;
use starcoin_rpc_api::miner::MinerApi;
use starcoin_service_registry::ServiceRef;
use starcoin_types::block::BlockHeaderExtra;
use starcoin_types::system_events::SubmitSealEvent;
use std::convert::TryInto;

pub struct MinerRpcImpl {
    miner_service: ServiceRef<MinerService>,
}

impl MinerRpcImpl {
    pub fn new(miner_service: ServiceRef<MinerService>) -> Self {
        Self { miner_service }
    }
}

impl MinerApi for MinerRpcImpl {
    fn submit(&self, minting_blob: String, nonce: u32, extra: String) -> Result<()> {
        let minting_blob = hex::decode(minting_blob).map_err(|e| RpcError::from(e).into())?;
        let e = hex::decode(extra).map_err(|e| RpcError::from(e).into())?;
        let e: Box<[u8; 4]> = e.into_boxed_slice().try_into().map_err(|_| {
            RpcError(jsonrpc_core::Error {
                code: ErrorCode::InvalidParams,
                message: "Invalid size for extra".to_string(),
                data: None,
            })
            .into()
        })?;
        let extra = BlockHeaderExtra::new(*e);
        self.miner_service
            .notify(SubmitSealEvent {
                nonce,
                extra,
                minting_blob,
            })
            .map_err(|e| map_err(e.into()))
    }
}
