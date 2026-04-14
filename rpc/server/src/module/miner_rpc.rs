// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use jsonrpsee::core::{async_trait, RpcResult};
use starcoin_miner::{MinerService, SubmitSealRequest, UpdateSubscriberNumRequest};
use starcoin_rpc_api::miner::MinerApiServer;
use starcoin_rpc_api::types::MintedBlockView;
use starcoin_service_registry::ServiceRef;
use starcoin_types::block::BlockHeaderExtra;
use starcoin_types::system_events::MintBlockEvent;
use std::convert::TryInto;

pub struct MinerRpcImpl {
    miner_service: ServiceRef<MinerService>,
}

impl MinerRpcImpl {
    pub fn new(miner_service: ServiceRef<MinerService>) -> Self {
        Self { miner_service }
    }
}

#[async_trait]
impl MinerApiServer for MinerRpcImpl {
    async fn submit(
        &self,
        minting_blob: String,
        nonce: u32,
        extra: String,
    ) -> RpcResult<MintedBlockView> {
        let miner_service = self.miner_service.clone();
        let minting_blob =
            hex::decode(minting_blob).map_err(|e| crate::module::map_jsonrpc_err(e.into()))?;
        let e: Box<[u8; 4]> = hex::decode(extra)
            .map_err(|e| crate::module::map_jsonrpc_err(e.into()))
            .and_then(|b| {
                b.into_boxed_slice().try_into().map_err(|_| {
                    crate::module::map_jsonrpc_err(anyhow::anyhow!("Invalid length of extra"))
                })
            })?;
        let extra = BlockHeaderExtra::new(*e);
        let block_hash = miner_service
            .send(SubmitSealRequest {
                nonce,
                extra,
                minting_blob,
            })
            .await
            .map_err(crate::module::map_jsonrpc_err)?
            .map_err(crate::module::map_jsonrpc_err)?;
        Ok(MintedBlockView { block_hash })
    }

    async fn get_job(&self) -> RpcResult<Option<MintBlockEvent>> {
        let miner_service = self.miner_service.clone();
        miner_service
            .send(UpdateSubscriberNumRequest { number: None })
            .await
            .map_err(crate::module::map_jsonrpc_err)
    }
}
