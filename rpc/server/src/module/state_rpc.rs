// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::module::map_err;
use bcs_ext::BCSCodec;
use futures::future::TryFutureExt;
use futures::FutureExt;
use starcoin_crypto::HashValue;
use starcoin_resource_viewer::MoveValueAnnotator;
use starcoin_rpc_api::state::StateApi;
use starcoin_rpc_api::types::{
    AccountStateSetView, AnnotatedMoveStructView, StrView, StructTagView,
};
use starcoin_rpc_api::FutureResult;
use starcoin_state_api::{ChainStateAsyncService, StateWithProof};
use starcoin_state_tree::StateNodeStore;
use starcoin_statedb::ChainStateDB;
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
        Box::new(fut.compat())
    }

    fn get_with_proof(&self, access_path: AccessPath) -> FutureResult<StateWithProof> {
        let fut = self
            .service
            .clone()
            .get_with_proof(access_path)
            .map_err(map_err);
        Box::new(fut.compat())
    }

    fn get_account_state(&self, address: AccountAddress) -> FutureResult<Option<AccountState>> {
        let fut = self
            .service
            .clone()
            .get_account_state(address)
            .map_err(map_err);
        Box::new(fut.compat())
    }

    fn get_account_state_set(
        &self,
        address: AccountAddress,
    ) -> FutureResult<Option<AccountStateSetView>> {
        let state_service = self.service.clone();
        let db = self.state_store.clone();
        let fut = async move {
            let state = state_service
                .clone()
                .get_account_state_set(address, None)
                .await?;
            let state_root = state_service.state_root().await?;
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
        Box::new(fut.map_err(map_err).boxed().compat())
    }

    fn get_state_root(&self) -> FutureResult<HashValue> {
        let fut = self.service.clone().state_root().map_err(map_err);
        Box::new(fut.compat())
    }

    fn get_with_proof_by_root(
        &self,
        access_path: AccessPath,
        state_root: HashValue,
    ) -> FutureResult<StateWithProof> {
        let fut = self
            .service
            .clone()
            .get_with_proof_by_root(access_path, state_root)
            .map_err(map_err);
        Box::new(fut.compat())
    }
}
