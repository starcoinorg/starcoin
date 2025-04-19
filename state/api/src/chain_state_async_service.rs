// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::message::StateRequestVMType;
use crate::{StateWithProof, StateWithTableItemProof};
use serde::de::DeserializeOwned;
use starcoin_crypto::HashValue;
use starcoin_types::{
    account_address::AccountAddress, account_state::AccountState, state_set::AccountStateSet,
};
use starcoin_vm_types::{
    access_path::AccessPath,
    move_resource::MoveResource,
    state_store::table::{TableHandle, TableInfo},
};

#[async_trait::async_trait]
pub trait ChainStateAsyncService: Clone + Unpin + Send + Sync {
    async fn get(
        self,
        access_path: AccessPath,
        vm_type: StateRequestVMType,
    ) -> anyhow::Result<Option<Vec<u8>>>;

    async fn get_with_proof(
        self,
        access_path: AccessPath,
        vm_type: StateRequestVMType,
    ) -> anyhow::Result<StateWithProof>;

    async fn get_resource<R>(
        self,
        address: AccountAddress,
        vm_type: StateRequestVMType,
    ) -> anyhow::Result<Option<R>>
    where
        R: MoveResource + DeserializeOwned,
    {
        let access_path = AccessPath::new(address, R::resource_path());
        let r = self
            .get(access_path, vm_type)
            .await
            .and_then(|state| match state {
                Some(state) => Ok(Some(bcs_ext::from_bytes::<R>(state.as_slice())?)),
                None => Ok(None),
            })?;
        Ok(r)
    }

    async fn get_account_state(
        self,
        address: AccountAddress,
        vm_type: StateRequestVMType,
    ) -> anyhow::Result<Option<AccountState>>;

    /// get account stateset on state_root(if empty, use current state root).
    async fn get_account_state_set(
        self,
        address: AccountAddress,
        state_root: Option<HashValue>,
        vm_type: StateRequestVMType,
    ) -> anyhow::Result<Option<AccountStateSet>>;

    async fn state_root(self, vm_type: StateRequestVMType) -> anyhow::Result<HashValue>;

    async fn get_with_proof_by_root(
        self,
        access_path: AccessPath,
        state_root: HashValue,
        vm_type: StateRequestVMType,
    ) -> anyhow::Result<StateWithProof>;

    async fn get_account_state_by_root(
        self,
        address: AccountAddress,
        state_root: HashValue,
        vm_type: StateRequestVMType,
    ) -> anyhow::Result<Option<AccountState>>;

    async fn get_with_table_item_proof(
        self,
        handle: TableHandle,
        key: Vec<u8>,
        vm_type: StateRequestVMType,
    ) -> anyhow::Result<StateWithTableItemProof>;
    async fn get_with_table_item_proof_by_root(
        self,
        handle: TableHandle,
        key: Vec<u8>,
        state_root: HashValue,
        vm_type: StateRequestVMType,
    ) -> anyhow::Result<StateWithTableItemProof>;

    async fn get_table_info(
        self,
        address: AccountAddress,
        vm_type: StateRequestVMType,
    ) -> anyhow::Result<Option<TableInfo>>;
}
