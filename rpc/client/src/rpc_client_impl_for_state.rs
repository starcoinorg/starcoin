// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use crate::{map_err, RpcClient};
use starcoin_crypto::HashValue;
use starcoin_rpc_api::types::state_api_types::{
    GetCodeOption, GetResourceOption, ListCodeOption, ListResourceOption, VmType,
};
use starcoin_rpc_api::types::{
    AccountStateSetView, CodeView, ListCodeView, ListResourceView, ResourceView,
    StateWithProofView, StateWithTableItemProofView, StrView, StructTagView, TableInfoView,
};
use starcoin_types::{
    account_address::AccountAddress,
    account_state::AccountState,
    language_storage::{ModuleId, StructTag},
};
use starcoin_vm_types::{access_path::AccessPath, state_store::table::TableHandle};

/// The rpc client implements some functions of the state
impl RpcClient {
    pub fn state_get(
        &self,
        access_path: AccessPath,
        vm_type: Option<VmType>,
    ) -> anyhow::Result<Option<Vec<u8>>> {
        self.call_rpc_blocking(|inner| inner.state_client.get(access_path, vm_type))
            .map_err(map_err)
    }

    pub fn state_get_with_proof(
        &self,
        access_path: AccessPath,
        vm_type: Option<VmType>,
    ) -> anyhow::Result<StateWithProofView> {
        self.call_rpc_blocking(|inner| inner.state_client.get_with_proof(access_path, vm_type))
            .map_err(map_err)
    }

    pub fn state_get_with_proof_by_root(
        &self,
        access_path: AccessPath,
        state_root: HashValue,
        vm_type: Option<VmType>,
    ) -> anyhow::Result<StateWithProofView> {
        self.call_rpc_blocking(|inner| {
            inner
                .state_client
                .get_with_proof_by_root(access_path, state_root, vm_type)
        })
        .map_err(map_err)
    }

    pub fn state_get_with_proof_by_root_raw(
        &self,
        access_path: AccessPath,
        state_root: HashValue,
        vm_type: Option<VmType>,
    ) -> anyhow::Result<StrView<Vec<u8>>> {
        self.call_rpc_blocking(|inner| {
            inner
                .state_client
                .get_with_proof_by_root_raw(access_path, state_root, vm_type)
        })
        .map_err(map_err)
    }

    pub fn state_get_state_root(&self, vm_type: Option<VmType>) -> anyhow::Result<HashValue> {
        self.call_rpc_blocking(|inner| inner.state_client.get_state_root(vm_type))
            .map_err(map_err)
    }

    pub fn state_get_account_state(
        &self,
        address: AccountAddress,
        vm_type: Option<VmType>,
    ) -> anyhow::Result<Option<AccountState>> {
        self.call_rpc_blocking(|inner| inner.state_client.get_account_state(address, vm_type))
            .map_err(map_err)
    }

    pub fn state_get_account_state_set(
        &self,
        address: AccountAddress,
        state_root: Option<HashValue>,
        vm_type: Option<VmType>,
    ) -> anyhow::Result<Option<AccountStateSetView>> {
        self.call_rpc_blocking(|inner| {
            inner
                .state_client
                .get_account_state_set(address, state_root, vm_type)
        })
        .map_err(map_err)
    }

    pub fn state_get_resource(
        &self,
        address: AccountAddress,
        resource_type: StructTag,
        decode: bool,
        state_root: Option<HashValue>,
        vm_type: Option<VmType>,
    ) -> anyhow::Result<Option<ResourceView>> {
        self.call_rpc_blocking(|inner| {
            inner.state_client.get_resource(
                address,
                StrView(resource_type),
                Some(GetResourceOption { decode, state_root }),
                vm_type,
            )
        })
        .map_err(map_err)
    }

    pub fn state_list_resource(
        &self,
        address: AccountAddress,
        decode: bool,
        state_root: Option<HashValue>,
        start_index: usize,
        max_size: usize,
        resource_types: Option<Vec<StructTagView>>,
        vm_type: Option<VmType>,
    ) -> anyhow::Result<ListResourceView> {
        self.call_rpc_blocking(|inner| {
            inner.state_client.list_resource(
                address,
                Some(ListResourceOption {
                    decode,
                    state_root,
                    start_index,
                    max_size,
                    resource_types,
                }),
                vm_type,
            )
        })
        .map_err(map_err)
    }

    pub fn state_get_code(
        &self,
        module_id: ModuleId,
        resolve: bool,
        state_root: Option<HashValue>,
        vm_type: Option<VmType>,
    ) -> anyhow::Result<Option<CodeView>> {
        self.call_rpc_blocking(|inner| {
            inner.state_client.get_code(
                StrView(module_id),
                Some(GetCodeOption {
                    resolve,
                    state_root,
                }),
                vm_type,
            )
        })
        .map_err(map_err)
    }

    pub fn state_list_code(
        &self,
        address: AccountAddress,
        resolve: bool,
        state_root: Option<HashValue>,
        vm_type: Option<VmType>,
    ) -> anyhow::Result<ListCodeView> {
        self.call_rpc_blocking(|inner| {
            inner.state_client.list_code(
                address,
                Some(ListCodeOption {
                    resolve,
                    state_root,
                }),
                vm_type,
            )
        })
        .map_err(map_err)
    }

    pub fn state_get_with_table_item_proof_by_root(
        &self,
        handle: TableHandle,
        key: Vec<u8>,
        state_root: HashValue,
        vm_type: Option<VmType>,
    ) -> anyhow::Result<StateWithTableItemProofView> {
        self.call_rpc_blocking(|inner| {
            inner
                .state_client
                .get_with_table_item_proof_by_root(handle, key, state_root, vm_type)
        })
        .map_err(map_err)
    }

    pub fn state_get_table_info(
        &self,
        address: AccountAddress,
        vm_type: Option<VmType>,
    ) -> anyhow::Result<Option<TableInfoView>> {
        self.call_rpc_blocking(|inner| inner.state_client.get_table_info(address, vm_type))
            .map_err(map_err)
    }

    pub fn get_state_node_by_node_hash(
        &self,
        key_hash: HashValue,
        vm_type: Option<VmType>,
    ) -> anyhow::Result<Option<Vec<u8>>> {
        self.call_rpc_blocking(|inner| {
            inner
                .state_client
                .get_state_node_by_node_hash(key_hash, vm_type)
        })
        .map_err(map_err)
    }
}
