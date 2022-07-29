// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::module::map_err;
use bcs_ext::BCSCodec;
use futures::future::TryFutureExt;
use futures::FutureExt;
use serde::Serialize;
use starcoin_abi_resolver::ABIResolver;
use starcoin_crypto::HashValue;
use starcoin_dev::playground::view_resource;
use starcoin_resource_viewer::MoveValueAnnotator;
use starcoin_rpc_api::state::{
    GetCodeOption, GetResourceOption, ListCodeOption, ListResourceOption, StateApi,
};
use starcoin_rpc_api::types::{
    AccountStateSetView, AnnotatedMoveStructView, CodeView, ListCodeView, ListResourceView,
    ResourceView, StateWithProofView, StrView, StructTagView,
};
use starcoin_rpc_api::FutureResult;
use starcoin_state_api::{ChainStateAsyncService, StateView};
use starcoin_state_tree::StateNodeStore;
use starcoin_statedb::{ChainStateDB, ChainStateReader};
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

    fn get_with_proof_raw(&self, access_path: AccessPath) -> FutureResult<StrView<Vec<u8>>> {
        let fut = self
            .service
            .clone()
            .get_with_proof(access_path)
            .map_ok(|p| {
                StrView(bcs_ext::to_bytes(&p).expect("Serialize StateWithProof should success."))
            })
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

    fn get_with_proof_by_root_raw(
        &self,
        access_path: AccessPath,
        state_root: HashValue,
    ) -> FutureResult<StrView<Vec<u8>>> {
        let fut = self
            .service
            .clone()
            .get_with_proof_by_root(access_path, state_root)
            .map_ok(|p| {
                StrView(bcs_ext::to_bytes(&p).expect("Serialize StateWithProof should success."))
            })
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
            let code = chain_state.get(&AccessPath::from(&module_id.0))?;
            Ok(match code {
                None => None,
                Some(c) => {
                    let abi = if option.resolve {
                        Some(ABIResolver::new(&chain_state).resolve_module(&module_id.0)?)
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
            let state_root = option
                .state_root
                .unwrap_or(service.clone().state_root().await?);
            let chain_state = ChainStateDB::new(state_store, Some(state_root));
            let data = chain_state.get(&AccessPath::resource_access_path(
                addr,
                resource_type.0.clone(),
            ))?;
            Ok(match data {
                None => None,
                Some(d) => {
                    let decoded = if option.decode {
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
            //TODO implement list state by iter, and pagination
            let state = statedb.get_account_state_set(&addr)?;
            match state {
                None => Ok(ListResourceView::default()),
                Some(s) => {
                    let resources: Result<BTreeMap<StructTagView, ResourceView>, anyhow::Error> = s
                        .resource_set()
                        .cloned()
                        .unwrap_or_default()
                        .iter()
                        .filter(|(k, v)| {
                            let struct_tag = StructTag::decode(k.as_slice()).unwrap();
                            //                         s = format!("{{ \"sec\": {} \"usec\": {} }}{}",
                            //     pt.sec,
                            //     pt.usec,
                            //     if isLast { "" } else { "," }
                            // )

                            // let resource_type_str = serde_json::to_string(&struct_tag.address)
                            //     .unwrap().to_owned()
                            //     + serde_json::to_string(&struct_tag.module).unwrap().to_owned()
                            //     + serde_json::to_string(&struct_tag.name).unwrap();
                            let struct_tag_value = serde_json::to_value(struct_tag).unwrap();
                            let address_value = struct_tag_value["address"].as_str().unwrap();
                            let module_value = struct_tag_value["module"].as_str().unwrap();
                            let name_value = struct_tag_value["name"].as_str().unwrap();
                            // struct_tag_value["name"];
                            let resource_type_value_str = format!(
                                "{}::{}::{}",
                                address_value,
                                module_value,
                                name_value
                            );
                            println!("resource_type_value_str: {}", resource_type_value_str);
                            
                            // let resource_type_str = format!(
                            //     "{}{}{}",
                            //     serde_json::to_string(&struct_tag.address).unwrap().as_str(),
                            //     serde_json::to_string(&struct_tag.module).unwrap().as_str(),
                            //     serde_json::to_string(&struct_tag.name).unwrap().as_str()
                            // );
                            // println!("address value is: {}", serde_json::to_string(&struct_tag.address).unwrap());
                            // for (address, module, name) in struct_tag {
                            //     serde_json::to_string(&struct_tag.address)
                            // }
                            // struct_tag.address + struct_tag.module + struct_tag.name;
                            // let str_view_struct_tag = StrView(struct_tag);
                            // let struct_tag_str =
                            //     serde_json::to_string(&str_view_struct_tag).unwrap();
                            println!(
                                // "struct_tag {:?}, {}, {:?}, {}, {}",
                                // "struct_tag {:?}, {}, {}, {}",
                                "struct_tag {}, {}",
                                // option.types.as_ref().unwrap().contains(&struct_tag_str),
                                // option.types.as_ref().unwrap().contains(&resource_type_str),
                                option.types.as_ref().unwrap().contains(&resource_type_value_str),
                                // str_view_struct_tag,
                                // struct_tag_str,
                                // resource_type_str,
                                resource_type_value_str
                            );
                            true
                            // option.types.contains()
                            // let id = Identifier::from_utf8(*k);
                            // println!("id = {}", id.);
                            // struct_tag.unwrap().serialize();
                            // let str = view_resource(&statedb, struct_tag.clone(), v.as_slice())
                            //     .ok()
                            //     .map(Into::into);
                            // println!("")
                            // match option.types {
                            //     None => true,
                            //     Some(types) => {
                            //         let struct_tag = StructTag::decode(k.as_slice())?;
                            //         let str = view_resource(&statedb, struct_tag.clone(), v.as_slice())
                            //             .ok()
                            //             .map(Into::into);
                            //         let decoded = if option.decode {
                            //             //ignore the resource decode error
                            //         } else {
                            //             None
                            //         };
                            //         let account_state: AccountState = account_state_bytes.as_slice().try_into()?;
                            //         account_state.

                            //         if types.contains(address) {

                            //         }
                            //     }
                            // }
                            // if option.types.is
                            // let account: AccountAddress = bcs_ext::from_bytes(address_bytes);
                            // let account_state: AccountState =
                            //     account_state_bytes.as_slice().try_into()?;
                            // let resource_root =
                            //     account_state.storage_roots()[DataType::RESOURCE.storage_index()];
                            // let resource = match resource_root {
                            //     None => None,
                            //     Some(root) => {
                            //         let account_tree =
                            //             StateTree::<StructTag>::new(storage.clone(), Some(root));
                            //         let data = account_tree.get(&resource_struct_tag)?;

                            //         if let Some(d) = data {
                            //             let annotated_struct = value_annotator.view_struct(
                            //                 resource_struct_tag.clone(),
                            //                 d.as_slice(),
                            //             )?;
                            //             let resource = annotated_struct;
                            //             let resource_json_value =
                            //                 serde_json::to_value(MoveStruct(resource))?;
                            //             Some(resource_json_value)
                            //         } else {
                            //             None
                            //         }
                            //     }
                            // };
                            // println!("x: {}", x.)
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
