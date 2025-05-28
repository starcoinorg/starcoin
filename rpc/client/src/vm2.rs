// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use crate::{map_err, remote_state_reader2::RemoteStateReader, RpcClient, StateRootOption};
use bcs_ext::BCSCodec;
use starcoin_rpc_api::chain::GetEventOption;
use starcoin_vm2_abi_types::{FunctionABI, ModuleABI, StructInstantiation};
use starcoin_vm2_account_api::AccountInfo;
use starcoin_vm2_crypto::HashValue;
use starcoin_vm2_rpc_api::{
    state_api::{GetCodeOption, GetResourceOption, ListCodeOption, ListResourceOption},
    DecodedMoveValue,
};
use starcoin_vm2_types::{
    account_address::AccountAddress,
    account_state::AccountState,
    language_storage::{ModuleId, StructTag},
    view::{
        AccountStateSetView, AnnotatedMoveStructView, CodeView, ContractCall, DryRunOutputView,
        DryRunTransactionRequest, FunctionIdView, ListCodeView, ListResourceView, ModuleIdView,
        ResourceView, SignedMessageView, StateWithProofView, StateWithTableItemProofView, StrView,
        StructTagView, TableInfoView, TransactionEventResponse, TransactionRequest,
    },
};
use starcoin_vm2_vm_types::{
    account_config::token_code::TokenCode,
    sign_message::SigningMessage,
    state_store::{state_key::StateKey, table::TableHandle},
    transaction::{DryRunTransaction, RawUserTransaction, SignedUserTransaction},
};

impl RpcClient {
    pub fn account_default2(&self) -> anyhow::Result<Option<AccountInfo>> {
        self.call_rpc_blocking(|inner| inner.account_client2.default())
            .map_err(map_err)
    }

    pub fn set_default_account2(&self, addr: AccountAddress) -> anyhow::Result<AccountInfo> {
        self.call_rpc_blocking(|inner| inner.account_client2.set_default_account(addr))
            .map_err(map_err)
    }

    pub fn account_create2(&self, password: String) -> anyhow::Result<AccountInfo> {
        self.call_rpc_blocking(|inner| inner.account_client2.create(password))
            .map_err(map_err)
    }

    pub fn account_list2(&self) -> anyhow::Result<Vec<AccountInfo>> {
        self.call_rpc_blocking(|inner| inner.account_client2.list())
            .map_err(map_err)
    }

    pub fn account_get2(&self, address: AccountAddress) -> anyhow::Result<Option<AccountInfo>> {
        self.call_rpc_blocking(|inner| inner.account_client2.get(address))
            .map_err(map_err)
    }

    /// partial sign a multisig account's txn
    pub fn account_sign_multisig_txn2(
        &self,
        raw_txn: RawUserTransaction,
        signer_address: AccountAddress,
    ) -> anyhow::Result<SignedUserTransaction> {
        self.call_rpc_blocking(|inner| inner.account_client2.sign_txn(raw_txn, signer_address))
            .map_err(map_err)
    }

    pub fn account_sign_txn_request2(
        &self,
        txn_request: TransactionRequest,
    ) -> anyhow::Result<SignedUserTransaction> {
        self.call_rpc_blocking(|inner| inner.account_client2.sign_txn_request(txn_request))
            .map_err(map_err)
            .and_then(|d: String| {
                hex::decode(d.as_str().strip_prefix("0x").unwrap_or(d.as_str()))
                    .map_err(anyhow::Error::new)
                    .and_then(|d| bcs_ext::from_bytes::<SignedUserTransaction>(d.as_slice()))
            })
    }

    pub fn account_sign_txn2(
        &self,
        raw_txn: RawUserTransaction,
    ) -> anyhow::Result<SignedUserTransaction> {
        let signer = raw_txn.sender();
        self.call_rpc_blocking(|inner| inner.account_client2.sign_txn(raw_txn, signer))
            .map_err(map_err)
    }

    pub fn account_sign_message2(
        &self,
        signer: AccountAddress,
        message: SigningMessage,
    ) -> anyhow::Result<SignedMessageView> {
        self.call_rpc_blocking(|inner| inner.account_client2.sign(signer, message))
            .map_err(map_err)
    }

    pub fn account_change_password2(
        &self,
        address: AccountAddress,
        new_password: String,
    ) -> anyhow::Result<AccountInfo> {
        self.call_rpc_blocking(|inner| {
            inner
                .account_client2
                .change_account_password(address, new_password)
        })
        .map_err(map_err)
    }

    pub fn account_lock2(&self, address: AccountAddress) -> anyhow::Result<AccountInfo> {
        self.call_rpc_blocking(|inner| inner.account_client2.lock(address))
            .map_err(map_err)
    }
    pub fn account_unlock2(
        &self,
        address: AccountAddress,
        password: String,
        duration: std::time::Duration,
    ) -> anyhow::Result<AccountInfo> {
        self.call_rpc_blocking(|inner| {
            inner
                .account_client2
                .unlock(address, password, Some(duration.as_secs() as u32))
        })
        .map_err(map_err)
    }
    pub fn account_export2(
        &self,
        address: AccountAddress,
        password: String,
    ) -> anyhow::Result<Vec<u8>> {
        self.call_rpc_blocking(|inner| inner.account_client2.export(address, password))
            .map_err(map_err)
    }
    pub fn account_import2(
        &self,
        address: AccountAddress,
        private_key: Vec<u8>,
        password: String,
    ) -> anyhow::Result<AccountInfo> {
        self.call_rpc_blocking(|inner| {
            inner
                .account_client2
                .import(address, StrView(private_key), password)
        })
        .map_err(map_err)
    }

    pub fn account_import_readonly2(
        &self,
        address: AccountAddress,
        public_key: Vec<u8>,
    ) -> anyhow::Result<AccountInfo> {
        self.call_rpc_blocking(|inner| {
            inner
                .account_client2
                .import_readonly(address, StrView(public_key))
        })
        .map_err(map_err)
    }

    pub fn account_accepted_tokens2(
        &self,
        address: AccountAddress,
    ) -> anyhow::Result<Vec<TokenCode>> {
        self.call_rpc_blocking(|inner| inner.account_client2.accepted_tokens(address))
            .map_err(map_err)
    }

    pub fn account_remove2(
        &self,
        address: AccountAddress,
        password: Option<String>,
    ) -> anyhow::Result<AccountInfo> {
        self.call_rpc_blocking(|inner| inner.account_client2.remove(address, password))
            .map_err(map_err)
    }

    // State client APIs.
    pub fn state_reader2(
        &self,
        state_root_opt: StateRootOption,
    ) -> anyhow::Result<RemoteStateReader> {
        RemoteStateReader::new(self, state_root_opt)
    }

    pub fn state_get2(&self, state_key: StateKey) -> anyhow::Result<Option<Vec<u8>>> {
        self.call_rpc_blocking(|inner| inner.state_client2.get(state_key))
            .map_err(map_err)
            .map(|v| v.map(|s| s.to_vec()))
    }

    pub fn state_get_with_proof2(&self, state_key: StateKey) -> anyhow::Result<StateWithProofView> {
        self.call_rpc_blocking(|inner| inner.state_client2.get_with_proof(state_key))
            .map_err(map_err)
    }

    pub fn state_get_with_proof_by_root2(
        &self,
        state_key: StateKey,
        state_root: HashValue,
    ) -> anyhow::Result<StateWithProofView> {
        self.call_rpc_blocking(|inner| {
            inner
                .state_client2
                .get_with_proof_by_root(state_key, state_root)
        })
        .map_err(map_err)
    }

    pub fn state_get_with_proof_by_root_raw2(
        &self,
        state_key: StateKey,
        state_root: HashValue,
    ) -> anyhow::Result<StrView<Vec<u8>>> {
        self.call_rpc_blocking(|inner| {
            inner
                .state_client2
                .get_with_proof_by_root_raw(state_key, state_root)
        })
        .map_err(map_err)
    }

    pub fn state_get_state_root2(&self) -> anyhow::Result<HashValue> {
        self.call_rpc_blocking(|inner| inner.state_client2.get_state_root())
            .map_err(map_err)
    }

    pub fn state_get_account_state2(
        &self,
        address: AccountAddress,
    ) -> anyhow::Result<AccountState> {
        self.call_rpc_blocking(|inner| inner.state_client2.get_account_state(address))
            .map_err(map_err)
    }

    pub fn state_get_account_state_set2(
        &self,
        address: AccountAddress,
        state_root: Option<HashValue>,
    ) -> anyhow::Result<Option<AccountStateSetView>> {
        self.call_rpc_blocking(|inner| {
            inner
                .state_client2
                .get_account_state_set(address, state_root)
        })
        .map_err(map_err)
    }

    pub fn state_get_resource2(
        &self,
        address: AccountAddress,
        resource_type: StructTag,
        decode: bool,
        state_root: Option<HashValue>,
    ) -> anyhow::Result<Option<ResourceView>> {
        self.call_rpc_blocking(|inner| {
            inner.state_client2.get_resource(
                address,
                StrView(resource_type),
                Some(GetResourceOption { decode, state_root }),
            )
        })
        .map_err(map_err)
    }

    pub fn state_list_resource2(
        &self,
        address: AccountAddress,
        decode: bool,
        state_root: Option<HashValue>,
        start_index: usize,
        max_size: usize,
        resource_types: Option<Vec<StructTagView>>,
    ) -> anyhow::Result<ListResourceView> {
        self.call_rpc_blocking(|inner| {
            inner.state_client2.list_resource(
                address,
                Some(ListResourceOption {
                    decode,
                    state_root,
                    start_index,
                    max_size,
                    resource_types,
                }),
            )
        })
        .map_err(map_err)
    }

    pub fn state_get_code2(
        &self,
        module_id: ModuleId,
        resolve: bool,
        state_root: Option<HashValue>,
    ) -> anyhow::Result<Option<CodeView>> {
        self.call_rpc_blocking(|inner| {
            inner.state_client2.get_code(
                StrView(module_id),
                Some(GetCodeOption {
                    resolve,
                    state_root,
                }),
            )
        })
        .map_err(map_err)
    }

    pub fn state_list_code2(
        &self,
        address: AccountAddress,
        resolve: bool,
        state_root: Option<HashValue>,
    ) -> anyhow::Result<ListCodeView> {
        self.call_rpc_blocking(|inner| {
            inner.state_client2.list_code(
                address,
                Some(ListCodeOption {
                    resolve,
                    state_root,
                }),
            )
        })
        .map_err(map_err)
    }

    pub fn state_get_with_table_item_proof_by_root2(
        &self,
        handle: TableHandle,
        key: Vec<u8>,
        state_root: HashValue,
    ) -> anyhow::Result<StateWithTableItemProofView> {
        self.call_rpc_blocking(|inner| {
            inner
                .state_client2
                .get_with_table_item_proof_by_root(handle, key, state_root)
        })
        .map_err(map_err)
    }

    pub fn state_get_table_info2(&self, address: AccountAddress) -> anyhow::Result<TableInfoView> {
        self.call_rpc_blocking(|inner| inner.state_client2.get_table_info(address))
            .map_err(map_err)
    }

    pub fn get_state_node_by_node_hash2(
        &self,
        key_hash: HashValue,
    ) -> anyhow::Result<Option<Vec<u8>>> {
        self.call_rpc_blocking(|inner| inner.state_client2.get_state_node_by_node_hash(key_hash))
            .map_err(map_err)
            .map(|v| v.map(|bytes| bytes.to_vec()))
    }

    // Contract client APIs.
    pub fn get_code2(&self, module_id: ModuleId) -> anyhow::Result<Option<String>> {
        let result: Option<StrView<Vec<u8>>> = self
            .call_rpc_blocking(|inner| inner.contract_client2.get_code(StrView(module_id)))
            .map_err(map_err)?;
        Ok(result.map(|s| s.to_string()))
    }

    pub fn get_resource2(
        &self,
        addr: AccountAddress,
        resource_type: StructTag,
    ) -> anyhow::Result<Option<AnnotatedMoveStructView>> {
        self.call_rpc_blocking(|inner| {
            inner
                .contract_client2
                .get_resource(addr, StrView(resource_type))
        })
        .map_err(map_err)
    }

    pub fn contract_call2(&self, call: ContractCall) -> anyhow::Result<Vec<DecodedMoveValue>> {
        self.call_rpc_blocking(|inner| inner.contract_client2.call_v2(call))
            .map_err(map_err)
    }

    pub fn contract_resolve_function2(
        &self,
        function_id: FunctionIdView,
    ) -> anyhow::Result<FunctionABI> {
        self.call_rpc_blocking(|inner| inner.contract_client2.resolve_function(function_id))
            .map_err(map_err)
    }

    pub fn contract_resolve_struct2(
        &self,
        struct_tag: StructTagView,
    ) -> anyhow::Result<StructInstantiation> {
        self.call_rpc_blocking(|inner| inner.contract_client2.resolve_struct(struct_tag))
            .map_err(map_err)
    }

    pub fn contract_resolve_module2(&self, module_id: ModuleIdView) -> anyhow::Result<ModuleABI> {
        self.call_rpc_blocking(|inner| inner.contract_client2.resolve_module(module_id))
            .map_err(map_err)
    }

    pub fn dry_run2(&self, txn: DryRunTransactionRequest) -> anyhow::Result<DryRunOutputView> {
        self.call_rpc_blocking(|inner| inner.contract_client2.dry_run(txn))
            .map_err(map_err)
    }
    pub fn dry_run_raw2(&self, txn: DryRunTransaction) -> anyhow::Result<DryRunOutputView> {
        let DryRunTransaction {
            raw_txn,
            public_key,
        } = txn;
        let raw_txn = hex::encode(raw_txn.encode()?);
        self.call_rpc_blocking(|inner| {
            inner
                .contract_client2
                .dry_run_raw(raw_txn, StrView(public_key))
        })
        .map_err(map_err)
    }

    pub fn next_sequence_number2_in_txpool(
        &self,
        address: AccountAddress,
    ) -> anyhow::Result<Option<u64>> {
        self.call_rpc_blocking(|inner| inner.txpool_client.next_sequence_number2(address))
            .map_err(map_err)
    }

    pub fn chain_get_events_by_txn_hash2(
        &self,
        txn_hash: HashValue,
        option: Option<GetEventOption>,
    ) -> anyhow::Result<Vec<TransactionEventResponse>> {
        self.call_rpc_blocking(|inner| inner.chain_client.get_events_by_txn_hash2(txn_hash, option))
            .map_err(map_err)
    }
}
