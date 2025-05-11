// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::module::map_err;
use bcs_ext::BCSCodec;
use bytes::Bytes;
use futures::future::TryFutureExt;
use futures::FutureExt;
use starcoin_vm2_abi_resolver::ABIResolver;
use starcoin_crypto::HashValue;
use starcoin_dev::playground::view_resource;
use starcoin_resource_viewer::MoveValueAnnotator;
use starcoin_rpc_api::state::{
    GetCodeOption, GetResourceOption, ListCodeOption, ListResourceOption, StateApi,
};
use starcoin_rpc_api::types::{
    AccountStateSetView, AnnotatedMoveStructView, CodeView, ListCodeView, ListResourceView,
    ResourceView, StateWithProofView, StateWithTableItemProofView, StrView, StructTagView,
    TableInfoView,
};
use starcoin_vm2_rpc_api::{FutureResult, state_api::StateApi};
use starcoin_vm2_state_api::ChainStateAsyncService;
use starcoin_vm2_state_tree::StateNodeStore;
use starcoin_statedb::{ChainStateDB, ChainStateReader};
use starcoin_types::language_storage::ModuleId;
use starcoin_types::{account_address::AccountAddress, account_state::AccountState};
use starcoin_vm_types::identifier::Identifier;
use starcoin_vm_types::language_storage::{struct_tag_match, StructTag};
use starcoin_vm_types::state_store::state_key::StateKey;
use starcoin_vm_types::state_store::table::TableHandle;
use starcoin_vm_types::state_store::TStateView;
use std::collections::BTreeMap;
use std::sync::Arc;
use starcoin_vm2_types::view::{
    AccountStateSetView, CodeView, ListCodeView, ListResourceView, ResourceView,
    StateWithProofView, StateWithTableItemProofView, StrView, StructTagView, TableInfoView,
};
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
    fn get(&self, state_key: StateKey) -> FutureResult<Option<Bytes>> {
        let fut = self.service.clone().get(state_key).map_err(map_err);
        Box::pin(fut)
    }

    fn get_state_node_by_node_hash(&self, key_hash: HashValue) -> FutureResult<Option<Bytes>> {
        let state_store = self.state_store.clone();
        let f = async move {
            let node = state_store.get(&key_hash)?.map(|n| n.0);
            Ok(node.map(Bytes::from))
        };
        Box::pin(f.map_err(map_err).boxed())
    }

    fn get_with_proof(&self, state_key: StateKey) -> FutureResult<StateWithProofView> {
        let fut = self
            .service
            .clone()
            .get_with_proof(state_key)
            .map_ok(|p| p.into())
            .map_err(map_err);
        Box::pin(fut)
    }

    fn get_with_proof_raw(&self, state_key: StateKey) -> FutureResult<StrView<Vec<u8>>> {
        let fut = self
            .service
            .clone()
            .get_with_proof(state_key)
            .map_ok(|p| {
                StrView(bcs_ext::to_bytes(&p).expect("Serialize StateWithProof should success."))
            })
            .map_err(map_err);
        Box::pin(fut)
    }

    fn get_account_state(&self, address: AccountAddress) -> FutureResult<AccountState> {
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
            let state_root = state_root.unwrap_or(state_service.state_root().await?);
            let statedb = ChainStateDB::new(db, Some(state_root));
            let state = statedb.get_account_state_set(&address)?;
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
        state_key: StateKey,
        state_root: HashValue,
    ) -> FutureResult<StateWithProofView> {
        let fut = self
            .service
            .clone()
            .get_with_proof_by_root(state_key, state_root)
            .map_ok(|p| p.into())
            .map_err(map_err);
        Box::pin(fut)
    }

    fn get_with_proof_by_root_raw(
        &self,
        state_key: StateKey,
        state_root: HashValue,
    ) -> FutureResult<StrView<Vec<u8>>> {
        let fut = self
            .service
            .clone()
            .get_with_proof_by_root(state_key, state_root)
            .map_ok(|p| {
                StrView(bcs_ext::to_bytes(&p).expect("Serialize StateWithProof should success."))
            })
            .map_err(map_err);
        Box::pin(fut)
    }

    fn get_table_info(&self, address: AccountAddress) -> FutureResult<TableInfoView> {
        let fut = self
            .service
            .clone()
            .get_table_info(address)
            .map_ok(|v| v.into())
            .map_err(map_err);
        Box::pin(fut)
    }

    fn get_with_table_item_proof(
        &self,
        handle: TableHandle,
        key: Vec<u8>,
    ) -> FutureResult<StateWithTableItemProofView> {
        let fut = self
            .service
            .clone()
            .get_with_table_item_proof(handle, key)
            .map_ok(|p| p.into())
            .map_err(map_err);
        Box::pin(fut)
    }

    fn get_with_table_item_proof_by_root(
        &self,
        handle: TableHandle,
        key: Vec<u8>,
        state_root: HashValue,
    ) -> FutureResult<StateWithTableItemProofView> {
        let fut = self
            .service
            .clone()
            .get_with_table_item_proof_by_root(handle, key, state_root)
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
        let option = option.unwrap_or_default();
        let f = async move {
            let state_root = option
                .state_root
                .unwrap_or(service.clone().state_root().await?);
            let chain_state = ChainStateDB::new(state_store, Some(state_root));
            let state_key = StateKey::module_id(&module_id.0);
            let code = chain_state.get_state_value_bytes(&state_key)?;
            Ok(match code {
                None => None,
                Some(c) => {
                    let abi = if option.resolve {
                        Some(ABIResolver::new(&chain_state).resolve_module(&module_id.0)?)
                    } else {
                        None
                    };

                    Some(CodeView {
                        code: StrView(c.to_vec()),
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
            let state_root = option
                .state_root
                .unwrap_or(service.clone().state_root().await?);
            let chain_state = ChainStateDB::new(state_store, Some(state_root));
            let state_key = StateKey::resource(&addr, &resource_type.0)?;
            let data = chain_state.get_state_value_bytes(&state_key)?;
            Ok(match data {
                None => None,
                Some(d) => {
                    let decoded = if option.decode {
                        let value =
                            view_resource(&chain_state, resource_type.0, d.to_vec().as_slice())?;
                        Some(value.into())
                    } else {
                        None
                    };

                    Some(ResourceView {
                        raw: StrView(d.to_vec()),
                        json: decoded,
                    })
                }
            })
        };
        Box::pin(f.map_err(map_err).boxed())
    }

    fn list_resource(
        &self,
        addr: AccountAddress,
        option: Option<ListResourceOption>,
    ) -> FutureResult<ListResourceView> {
        let state_service = self.service.clone();
        let db = self.state_store.clone();
        let option = option.unwrap_or_default();
        let fut = async move {
            let state_root = option
                .state_root
                .unwrap_or(state_service.state_root().await?);
            let statedb = ChainStateDB::new(db, Some(state_root));

            let state = statedb.get_account_state_set(&addr)?;
            let filter_types = option.resource_types;
            if filter_types.is_some() && filter_types.as_ref().unwrap().len() > 10 {
                return Err(anyhow::anyhow!("Query resources is limited by 10"));
            }

            match state {
                None => Ok(ListResourceView::default()),
                Some(s) => {
                    let resources: Result<BTreeMap<StructTagView, ResourceView>, anyhow::Error> = s
                        .resource_set()
                        .cloned()
                        .unwrap_or_default()
                        .iter()
                        .filter(|(k, _)| {
                            if filter_types.is_none() {
                                return true;
                            }

                            let resource_struct_tag = StructTag::decode(k.as_slice()).unwrap();
                            for filter_type in filter_types.as_ref().unwrap() {
                                if struct_tag_match(&filter_type.0, &resource_struct_tag) {
                                    return true;
                                }
                            }
                            false
                        })
                        .skip(option.start_index)
                        .take(option.max_size)
                        .map(|(k, v)| {
                            let struct_tag = StructTag::decode(k.as_slice())?;
                            let decoded = if option.decode {
                                //ignore the resource decode error
                                view_resource(&statedb, struct_tag.clone(), v.as_slice())
                                    .ok()
                                    .map(Into::into)
                            } else {
                                None
                            };

                            Ok((
                                StrView(struct_tag),
                                ResourceView {
                                    raw: StrView(v.clone()),
                                    json: decoded,
                                },
                            ))
                        })
                        .collect();
                    Ok(ListResourceView {
                        resources: resources?,
                    })
                }
            }
        };
        Box::pin(fut.map_err(map_err).boxed())
    }

    fn list_code(
        &self,
        addr: AccountAddress,
        option: Option<ListCodeOption>,
    ) -> FutureResult<ListCodeView> {
        let state_service = self.service.clone();
        let db = self.state_store.clone();
        let option = option.unwrap_or_default();
        let fut = async move {
            let state_root = option
                .state_root
                .unwrap_or(state_service.state_root().await?);
            let statedb = ChainStateDB::new(db, Some(state_root));
            //TODO implement list state by iter, and pagination
            let state = statedb.get_account_state_set(&addr)?;
            match state {
                None => Ok(ListCodeView::default()),
                Some(s) => {
                    let codes: Result<BTreeMap<Identifier, CodeView>, anyhow::Error> = s
                        .code_set()
                        .cloned()
                        .unwrap_or_default()
                        .iter()
                        .map(|(k, v)| {
                            let identifier = Identifier::decode(k.as_slice())?;
                            let module_id = ModuleId::new(addr, identifier.clone());
                            let abi = if option.resolve {
                                //ignore the resolve error
                                ABIResolver::new(&statedb).resolve_module(&module_id).ok()
                            } else {
                                None
                            };

                            Ok((
                                identifier,
                                CodeView {
                                    code: StrView(v.clone()),
                                    abi,
                                },
                            ))
                        })
                        .collect();
                    Ok(ListCodeView { codes: codes? })
                }
            }
        };
        Box::pin(fut.map_err(map_err).boxed())
    }
}