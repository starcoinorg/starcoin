// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use crate::module::map_err;
use futures::{FutureExt, TryFutureExt};
use starcoin_miner::{MinerService, SubmitSealRequest, UpdateSubscriberNumRequest};
use starcoin_rpc_api::miner::MinerApi;
use starcoin_rpc_api::types::MintedBlockView;
use starcoin_rpc_api::FutureResult;
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

impl MinerApi for MinerRpcImpl {
    fn submit(
        &self,
        minting_blob: String,
        nonce: u32,
        extra: String,
    ) -> FutureResult<MintedBlockView> {
        let miner_service = self.miner_service.clone();
        let fut = async move {
            let minting_blob = hex::decode(minting_blob)?;
            let e: Box<[u8; 4]> = hex::decode(extra).map_err(|e| e.into()).and_then(|b| {
                b.into_boxed_slice()
                    .try_into()
                    .map_err(|_| anyhow::anyhow!("Invalid length of extra"))
            })?;
            let extra = BlockHeaderExtra::new(*e);
            let block_hash = miner_service
                .send(SubmitSealRequest {
                    nonce,
                    extra,
                    minting_blob,
                })
                .await??;
            Ok(MintedBlockView { block_hash })
        }
        .map_err(map_err);
        Box::pin(fut.boxed())
    }

    fn get_job(&self) -> FutureResult<Option<MintBlockEvent>> {
        let miner_service = self.miner_service.clone();
        let fut = async move {
            miner_service
                .send(UpdateSubscriberNumRequest { number: None })
                .await
        }
        .map_err(map_err);
        Box::pin(fut.boxed())
    }
}
