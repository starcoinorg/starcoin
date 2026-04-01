// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::map_err;
use anyhow::{bail, Result};
use bcs_ext::BCSCodec;
use bytes::Bytes;
use jsonrpsee::core::{async_trait, RpcResult};
use serde_json::Value;
use starcoin_vm2_abi_decoder::DecodedMoveValue;
use starcoin_vm2_abi_resolver::ABIResolver;
use starcoin_vm2_crypto::HashValue;
use starcoin_vm2_dev::playground::view_resource;
use starcoin_vm2_resource_viewer::MoveValueAnnotator;
use starcoin_vm2_vm_types::move_resource::MoveStructType;

use starcoin_vm2_rpc_api::state_api::{
    GetCodeOption, GetResourceOption, ListCodeOption, ListResourceOption, StateApiServer,
};
use starcoin_vm2_state_api::{ChainStateAsyncService, StateNodeStore, StateReaderExt};
use starcoin_vm2_statedb::{ChainStateDB, ChainStateReader};
use starcoin_vm2_types::view::{
    AccountStateSetView, AnnotatedMoveStructView, CodeView, ListCodeView, ListResourceView,
    ResourceView, StateWithProofView, StateWithTableItemProofView, StrView, StructTagView,
};
use starcoin_vm2_types::{account_address::AccountAddress, account_state::AccountState};
use starcoin_vm2_vm_types::{
    account_config::resources::{
        primary_store, ConcurrentFungibleBalanceResource, FungibleStoreResource,
        ObjectGroupResource,
    },
    identifier::Identifier,
    language_storage::{struct_tag_match, ModuleId, StructTag},
    on_chain_config::Features,
    state_store::{state_key::StateKey, table::TableHandle, TStateView},
    token::{stc::G_STC_TOKEN_CODE, token_code::TokenCode},
};
use std::{collections::BTreeMap, str::FromStr, sync::Arc};

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
    #[allow(dead_code)]
    pub fn new(service: S, state_store: Arc<dyn StateNodeStore>) -> Self {
        Self {
            service,
            state_store,
        }
    }
}

#[async_trait]
impl<S> StateApiServer for StateRpcImpl<S>
where
    S: ChainStateAsyncService,
{
    async fn get(&self, state_key: StateKey) -> RpcResult<Option<Bytes>> {
        self.service
            .clone()
            .get(state_key)
            .await
            .map_err(map_err)
            .map_err(crate::map_jsonrpc_err)
    }

    async fn get_state_node_by_node_hash(&self, key_hash: HashValue) -> RpcResult<Option<Bytes>> {
        let node = self
            .state_store
            .clone()
            .get(&key_hash)
            .map_err(map_err)
            .map_err(crate::map_jsonrpc_err)?
            .map(|n| n.0)
            .map(Bytes::from);
        Ok(node)
    }

    async fn get_with_proof(&self, state_key: StateKey) -> RpcResult<StateWithProofView> {
        self.service
            .clone()
            .get_with_proof(state_key)
            .await
            .map(|p| p.into())
            .map_err(map_err)
            .map_err(crate::map_jsonrpc_err)
    }

    async fn get_with_proof_raw(&self, state_key: StateKey) -> RpcResult<StrView<Vec<u8>>> {
        self.service
            .clone()
            .get_with_proof(state_key)
            .await
            .map(|p| {
                StrView(bcs_ext::to_bytes(&p).expect("Serialize StateWithProof should success."))
            })
            .map_err(map_err)
            .map_err(crate::map_jsonrpc_err)
    }

    async fn get_account_state(&self, address: AccountAddress) -> RpcResult<AccountState> {
        self.service
            .clone()
            .get_account_state(address)
            .await
            .map_err(map_err)
            .map_err(crate::map_jsonrpc_err)
    }

    async fn get_account_state_set(
        &self,
        address: AccountAddress,
        state_root: Option<HashValue>,
    ) -> RpcResult<Option<AccountStateSetView>> {
        let state_service = self.service.clone();
        let db = self.state_store.clone();
        async move {
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
        }
        .await
        .map_err(map_err)
        .map_err(crate::map_jsonrpc_err)
    }

    async fn get_state_root(&self) -> RpcResult<HashValue> {
        self.service
            .clone()
            .state_root()
            .await
            .map_err(map_err)
            .map_err(crate::map_jsonrpc_err)
    }

    async fn get_with_proof_by_root(
        &self,
        state_key: StateKey,
        state_root: HashValue,
    ) -> RpcResult<StateWithProofView> {
        self.service
            .clone()
            .get_with_proof_by_root(state_key, state_root)
            .await
            .map(|p| p.into())
            .map_err(map_err)
            .map_err(crate::map_jsonrpc_err)
    }

    async fn get_with_proof_by_root_raw(
        &self,
        state_key: StateKey,
        state_root: HashValue,
    ) -> RpcResult<StrView<Vec<u8>>> {
        self.service
            .clone()
            .get_with_proof_by_root(state_key, state_root)
            .await
            .map(|p| {
                StrView(bcs_ext::to_bytes(&p).expect("Serialize StateWithProof should success."))
            })
            .map_err(map_err)
            .map_err(crate::map_jsonrpc_err)
    }

    async fn get_with_table_item_proof(
        &self,
        handle: TableHandle,
        key: Vec<u8>,
    ) -> RpcResult<StateWithTableItemProofView> {
        self.service
            .clone()
            .get_with_table_item_proof(handle, key)
            .await
            .map(|p| p.into())
            .map_err(map_err)
            .map_err(crate::map_jsonrpc_err)
    }

    async fn get_with_table_item_proof_by_root(
        &self,
        handle: TableHandle,
        key: Vec<u8>,
        state_root: HashValue,
    ) -> RpcResult<StateWithTableItemProofView> {
        self.service
            .clone()
            .get_with_table_item_proof_by_root(handle, key, state_root)
            .await
            .map(|p| p.into())
            .map_err(map_err)
            .map_err(crate::map_jsonrpc_err)
    }

    async fn get_code(
        &self,
        module_id: StrView<ModuleId>,
        option: Option<GetCodeOption>,
    ) -> RpcResult<Option<CodeView>> {
        let service = self.service.clone();
        let state_store = self.state_store.clone();
        let option = option.unwrap_or_default();
        async move {
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
        }
        .await
        .map_err(map_err)
        .map_err(crate::map_jsonrpc_err)
    }

    async fn get_resource(
        &self,
        addr: AccountAddress,
        resource_type: StrView<StructTag>,
        option: Option<GetResourceOption>,
    ) -> RpcResult<Option<ResourceView>> {
        let service = self.service.clone();
        let state_store = self.state_store.clone();
        let option = option.unwrap_or_default();
        async move {
            let state_root = option
                .state_root
                .unwrap_or(service.clone().state_root().await?);
            let chain_state = ChainStateDB::new(state_store, Some(state_root));
            let primary_store_opt = option.primary_fungible_store.clone();
            let data = if let Some(primary_store) = option.primary_fungible_store {
                ensure_fungible_store_struct(&resource_type.0)?;
                resolve_primary_store_bytes(&chain_state, addr, primary_store.token_code)?
            } else {
                let state_key = StateKey::resource(&addr, &resource_type.0)?;
                chain_state
                    .get_state_value_bytes(&state_key)?
                    .map(|bytes| bytes.to_vec())
            };
            Ok(match data {
                None => None,
                Some(d) => {
                    let mut decoded = if option.decode {
                        let struct_tag = resource_type.0.clone();
                        let value = view_resource(&chain_state, struct_tag, d.to_vec().as_slice())?;
                        Some(value.into())
                    } else {
                        None
                    };
                    if let (Some(primary_store), Some(json)) = (primary_store_opt, decoded.as_mut())
                    {
                        if resource_type.0 == FungibleStoreResource::struct_tag() {
                            apply_primary_store_balance_override(
                                &chain_state,
                                addr,
                                primary_store.token_code,
                                d.as_slice(),
                                json,
                            )?;
                        }
                    }

                    Some(ResourceView {
                        raw: StrView(d.to_vec()),
                        json: decoded,
                    })
                }
            })
        }
        .await
        .map_err(map_err)
        .map_err(crate::map_jsonrpc_err)
    }

    async fn list_resource(
        &self,
        addr: AccountAddress,
        option: Option<ListResourceOption>,
    ) -> RpcResult<ListResourceView> {
        let state_service = self.service.clone();
        let db = self.state_store.clone();
        async move {
            let ListResourceOption {
                decode,
                state_root,
                start_index,
                max_size,
                resource_types,
                primary_fungible_store,
            } = option.unwrap_or_default();
            let state_root = state_root.unwrap_or(state_service.state_root().await?);
            let statedb = ChainStateDB::new(db, Some(state_root));

            let state = statedb.get_account_state_set(&addr)?;
            let filter_types = resource_types;
            if filter_types.is_some() && filter_types.as_ref().unwrap().len() > 10 {
                return Err(anyhow::anyhow!("Query resources is limited by 10"));
            }

            let matches_filter = |tag: &StructTag| {
                if let Some(filters) = &filter_types {
                    filters
                        .iter()
                        .any(|filter| struct_tag_match(&filter.0, tag))
                } else {
                    true
                }
            };

            let mut collected: Vec<(StructTag, Vec<u8>)> = match state {
                None => Vec::new(),
                Some(s) => {
                    let mut entries = Vec::new();
                    for (k, v) in s.resource_set().cloned().unwrap_or_default().iter() {
                        let struct_tag = StructTag::decode(k.as_slice())?;
                        if matches_filter(&struct_tag) {
                            entries.push((struct_tag, v.clone()));
                        }
                    }
                    entries
                }
            };

            if let Some(primary_store) = &primary_fungible_store {
                let primary_tag = FungibleStoreResource::struct_tag();
                if matches_filter(&primary_tag) {
                    if let Some(bytes) = resolve_primary_store_bytes(
                        &statedb,
                        addr,
                        primary_store.token_code.clone(),
                    )? {
                        collected.push((primary_tag, bytes));
                    }
                }
            }

            let resources: Result<BTreeMap<StructTagView, ResourceView>, anyhow::Error> = collected
                .into_iter()
                .skip(start_index)
                .take(max_size)
                .map(|(struct_tag, bytes)| {
                    let mut decoded = if decode {
                        view_resource(&statedb, struct_tag.clone(), bytes.as_slice())
                            .ok()
                            .map(Into::into)
                    } else {
                        None
                    };
                    if let (Some(primary_store), Some(json)) =
                        (primary_fungible_store.clone(), decoded.as_mut())
                    {
                        if struct_tag == FungibleStoreResource::struct_tag() {
                            apply_primary_store_balance_override(
                                &statedb,
                                addr,
                                primary_store.token_code,
                                bytes.as_slice(),
                                json,
                            )?;
                        }
                    }

                    Ok((
                        StrView(struct_tag),
                        ResourceView {
                            raw: StrView(bytes),
                            json: decoded,
                        },
                    ))
                })
                .collect();

            Ok(ListResourceView {
                resources: resources?,
            })
        }
        .await
        .map_err(map_err)
        .map_err(crate::map_jsonrpc_err)
    }

    async fn list_code(
        &self,
        addr: AccountAddress,
        option: Option<ListCodeOption>,
    ) -> RpcResult<ListCodeView> {
        let state_service = self.service.clone();
        let db = self.state_store.clone();
        let option = option.unwrap_or_default();
        async move {
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
        }
        .await
        .map_err(map_err)
        .map_err(crate::map_jsonrpc_err)
    }
}

fn ensure_fungible_store_struct(tag: &StructTag) -> Result<()> {
    if tag == &FungibleStoreResource::struct_tag() {
        Ok(())
    } else {
        bail!(
            "Primary fungible store option requires struct tag {}",
            FungibleStoreResource::struct_tag()
        )
    }
}

fn resolve_primary_store_bytes(
    chain_state: &ChainStateDB,
    owner: AccountAddress,
    token_code: Option<String>,
) -> Result<Option<Vec<u8>>> {
    let token_code = match token_code {
        Some(code) => TokenCode::from_str(&code)?,
        None => G_STC_TOKEN_CODE.clone(),
    };
    let derived = primary_store(&owner, &token_code.to_canonical_string())?;
    let group_key = StateKey::resource_group(&derived, &ObjectGroupResource::struct_tag());
    let split_enabled = chain_state
        .get_on_chain_config::<Features>()
        .map(|features| features.is_resource_groups_split_in_vm_change_set_enabled())
        .unwrap_or(false);
    Ok(chain_state
        .get_resource_group_struct_tag_bytes_with_flag(
            &owner,
            &group_key,
            &FungibleStoreResource::struct_tag(),
            split_enabled,
        )?
        .map(|bytes| bytes.to_vec()))
}

fn apply_primary_store_balance_override(
    chain_state: &ChainStateDB,
    owner: AccountAddress,
    token_code: Option<String>,
    _store_bytes: &[u8],
    json: &mut DecodedMoveValue,
) -> Result<()> {
    let token_code = match token_code {
        Some(code) => TokenCode::from_str(&code)?,
        None => G_STC_TOKEN_CODE.clone(),
    };
    let derived = primary_store(&owner, &token_code.to_canonical_string())?;
    let group_key = StateKey::resource_group(&derived, &ObjectGroupResource::struct_tag());
    let split_enabled = chain_state
        .get_on_chain_config::<Features>()
        .map(|features| features.is_resource_groups_split_in_vm_change_set_enabled())
        .unwrap_or(false);
    let bytes = chain_state.get_resource_group_struct_tag_bytes_with_flag(
        &owner,
        &group_key,
        &ConcurrentFungibleBalanceResource::struct_tag(),
        split_enabled,
    )?;
    let bytes = if let Some(bytes) = bytes {
        bytes
    } else {
        bail!(
            "ConcurrentFungibleBalance not found for primary store owner={}, token={}",
            owner,
            token_code.to_canonical_string()
        );
    };
    let concurrent = bcs_ext::from_bytes::<ConcurrentFungibleBalanceResource>(&bytes)?;
    let balance = concurrent.balance();

    if let Value::Object(map) = &mut json.0 {
        map.insert("balance".to_string(), Value::Number(balance.into()));
    }
    Ok(())
}
