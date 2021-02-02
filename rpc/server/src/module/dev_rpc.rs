// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::module::map_err;
use futures::future::TryFutureExt;
use futures::FutureExt;
use starcoin_dev::playground::PlaygroudService;
use starcoin_rpc_api::dev::DevApi;
use starcoin_rpc_api::types::{AnnotatedMoveValueView, ContractCall};
use starcoin_rpc_api::FutureResult;
use starcoin_state_api::ChainStateAsyncService;

pub struct DevRpcImpl<S>
where
    S: ChainStateAsyncService + 'static,
{
    service: S,
    playground: PlaygroudService,
}

impl<S> DevRpcImpl<S>
where
    S: ChainStateAsyncService,
{
    pub fn new(service: S, playground: PlaygroudService) -> Self {
        Self {
            service,
            playground,
        }
    }
}

impl<S> DevApi for DevRpcImpl<S>
where
    S: ChainStateAsyncService,
{
    fn call_contract(&self, call: ContractCall) -> FutureResult<Vec<AnnotatedMoveValueView>> {
        let service = self.service.clone();
        let playground = self.playground.clone();
        let ContractCall {
            module_address,
            module_name,
            func,
            type_args,
            args,
        } = call;
        let f = async move {
            let state_root = service.state_root().await?;
            let output = playground.call_contract(
                state_root,
                module_address,
                module_name,
                func,
                type_args.into_iter().map(|v| v.0).collect(),
                args.into_iter().map(|v| v.0).collect(),
            )?;
            Ok(output.into_iter().map(Into::into).collect())
        }
        .map_err(map_err);
        Box::pin(f.boxed())
    }
}
