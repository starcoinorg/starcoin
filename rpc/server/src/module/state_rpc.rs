// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::module::map_err;
use bcs_ext::BCSCodec;
use futures::future::TryFutureExt;
use futures::FutureExt;
use starcoin_crypto::HashValue;
use starcoin_dev::playground::view_resource;
use starcoin_resource_viewer::abi_resolver::ABIResolver;
use starcoin_resource_viewer::MoveValueAnnotator;
use starcoin_rpc_api::state::{GetCodeOption, GetResourceOption, StateApi};
use starcoin_rpc_api::types::{
    AccountStateSetView, AnnotatedMoveStructView, CodeView, ResourceView, StateWithProofView,
    StrView, StructTagView,
};
use starcoin_rpc_api::FutureResult;
use starcoin_state_api::ChainStateAsyncService;
use starcoin_state_tree::StateNodeStore;
use starcoin_statedb::ChainStateDB;
use starcoin_types::language_storage::ModuleId;
use starcoin_types::{
    access_path::AccessPath, account_address::AccountAddress, account_state::AccountState,
};
use starcoin_vm_types::identifier::Identifier;
use starcoin_vm_types::language_storage::StructTag;
use std::collections::BTreeMap;
use std::sync::Arc;

pub struct StateRpcImpl<S>
where
    S: ChainStateAsyncService + 'static,
{
    service: S,
    state_store: Arc<dyn StateNodeStore>,
}

impl<S> StateRpcImpl<S>
where
    S: ChainStateAsyncService,
{
    pub fn new(service: S, state_store: Arc<dyn StateNodeStore>) -> Self {
        Self {
            service,
            state_store,
        }
    }
}

impl<S> StateApi for StateRpcImpl<S>
where
    S: ChainStateAsyncService,
{
    fn get(&self, access_path: AccessPath) -> FutureResult<Option<Vec<u8>>> {
        let fut = self.service.clone().get(access_path).map_err(map_err);
        Box::pin(fut)
    }

    fn get_with_proof(&self, access_path: AccessPath) -> FutureResult<StateWithProofView> {
        let fut = self
            .service
            .clone()
            .get_with_proof(access_path)
            .map_ok(|p| p.into())
            .map_err(map_err);
        Box::pin(fut)
    }

    fn get_account_state(&self, address: AccountAddress) -> FutureResult<Option<AccountState>> {
        let fut = self
            .service
            .clone()
            .get_account_state(address)
            .map_err(map_err);
        Box::pin(fut)
    }

    fn get_account_state_set(
        &self,
        address: AccountAddress,
        state_root: Option<HashValue>,
    ) -> FutureResult<Option<AccountStateSetView>> {
        let state_service = self.service.clone();
        let db = self.state_store.clone();
        let fut = async move {
            let state = state_service
                .clone()
                .get_account_state_set(address, state_root)
                .await?;
            let state_root = state_root.unwrap_or(state_service.state_root().await?);
            let statedb = ChainStateDB::new(db, Some(state_root));
            let annotator = MoveValueAnnotator::new(&statedb);
            match state {
                None => Ok(None),
                Some(s) => {
                    let codes: Result<BTreeMap<Identifier, StrView<Vec<u8>>>, _> = s
                        .code_set()
                        .cloned()
                        .unwrap_or_default()
                        .iter()
                        .map(|(k, v)| {
                            Identifier::decode(k.as_slice()).map(|k| (k, StrView(v.clone())))
                        })
                        .collect();

                    let resources: Result<
                        BTreeMap<StructTagView, AnnotatedMoveStructView>,
                        anyhow::Error,
                    > = s
                        .resource_set()
                        .cloned()
                        .unwrap_or_default()
                        .iter()
                        .map(|(k, v)| {
                            let struct_tag = StructTag::decode(k.as_slice())?;
                            let struct_data =
                                annotator.view_struct(struct_tag.clone(), v.as_slice())?;
                            Ok((StrView(struct_tag), struct_data.into()))
                        })
                        .collect();
                    Ok(Some(AccountStateSetView {
                        codes: codes?,
                        resources: resources?,
                    }))
                }
            }
        };
        Box::pin(fut.map_err(map_err).boxed())
    }

    fn get_state_root(&self) -> FutureResult<HashValue> {
        let fut = self.service.clone().state_root().map_err(map_err);
        Box::pin(fut)
    }

    fn get_with_proof_by_root(
        &self,
        access_path: AccessPath,
        state_root: HashValue,
    ) -> FutureResult<StateWithProofView> {
        let fut = self
            .service
            .clone()
            .get_with_proof_by_root(access_path, state_root)
            .map_ok(|p| p.into())
            .map_err(map_err);
        Box::pin(fut)
    }

    fn get_code(
        &self,
        module_id: StrView<ModuleId>,
        option: Option<GetCodeOption>,
    ) -> FutureResult<Option<CodeView>> {
        let service = self.service.clone();
        let state_store = self.state_store.clone();
        let f = async move {
            let state_root = service.clone().state_root().await?;
            let code = service.get(AccessPath::from(&module_id.0)).await?;
            Ok(match code {
                None => None,
                Some(c) => {
                    let option = option.unwrap_or_default();
                    let abi = if option.resolve {
                        let state = ChainStateDB::new(state_store, Some(state_root));
                        Some(ABIResolver::new(&state).resolve_module(&module_id.0)?)
                    } else {
                        None
                    };

                    Some(CodeView {
                        code: StrView(c),
                        abi,
                    })
                }
            })
        };
        Box::pin(f.map_err(map_err).boxed())
    }

    fn get_resource(
        &self,
        addr: AccountAddress,
        resource_type: StrView<StructTag>,
        option: Option<GetResourceOption>,
    ) -> FutureResult<Option<ResourceView>> {
        let service = self.service.clone();
        let state_store = self.state_store.clone();
        let option = option.unwrap_or_default();
        let f = async move {
            let state_root = service.clone().state_root().await?;
            let data = service
                .get(AccessPath::resource_access_path(
                    addr,
                    resource_type.0.clone(),
                ))
                .await?;
            Ok(match data {
                None => None,
                Some(d) => {
                    let decoded = if option.decode {
                        let chain_state = ChainStateDB::new(state_store, Some(state_root));
                        let value = view_resource(&chain_state, resource_type.0, d.as_slice())?;
                        Some(value.into())
                    } else {
                        None
                    };

                    Some(ResourceView {
                        raw: StrView(d),
                        json: decoded,
                    })
                }
            })
        };
        Box::pin(f.map_err(map_err).boxed())
    }
}
