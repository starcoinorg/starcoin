// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::module::map_err;
use futures::future::TryFutureExt;
use futures::FutureExt;
use starcoin_dev::playground::PlaygroudService;
use starcoin_rpc_api::contract_api::ContractApi;
use starcoin_rpc_api::types::{AnnotatedMoveStruct, AnnotatedMoveValue, ContractCall, StrView};
use starcoin_rpc_api::FutureResult;
use starcoin_state_api::ChainStateAsyncService;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::language_storage::{ModuleId, StructTag};
use starcoin_vm_types::access_path::AccessPath;
use starcoin_vm_types::language_storage::ResourceKey;

pub struct ContractRpcImpl<S>
where
    S: ChainStateAsyncService + 'static,
{
    service: S,
    playground: PlaygroudService,
}

impl<S> ContractRpcImpl<S>
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

impl<S> ContractApi for ContractRpcImpl<S>
where
    S: ChainStateAsyncService,
{
    fn get_code(&self, module_id: StrView<ModuleId>) -> FutureResult<Option<StrView<Vec<u8>>>> {
        let service = self.service.clone();
        let f = async move {
            let code = service
                .get(AccessPath::code_access_path(&module_id.0))
                .await?;
            Ok(code.map(StrView))
        };
        Box::new(f.map_err(map_err).boxed().compat())
    }

    fn get_resource(
        &self,
        addr: AccountAddress,
        resource_type: StrView<StructTag>,
    ) -> FutureResult<Option<AnnotatedMoveStruct>> {
        let service = self.service.clone();
        let playground = self.playground.clone();
        let f = async move {
            let state_root = service.clone().state_root().await?;
            let data = service
                .get(AccessPath::resource_access_path(&ResourceKey::new(
                    addr,
                    resource_type.0.clone(),
                )))
                .await?;
            match data {
                None => Ok(None),
                Some(d) => {
                    let value =
                        playground.view_resource(state_root, &resource_type.0, d.as_slice())?;
                    Ok(Some(value))
                }
            }
        };
        Box::new(f.map_err(map_err).boxed().compat())
    }

    fn call(&self, call: ContractCall) -> FutureResult<Vec<AnnotatedMoveValue>> {
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
            Ok(output)
        }
        .map_err(map_err);
        Box::new(f.boxed().compat())
    }
}
