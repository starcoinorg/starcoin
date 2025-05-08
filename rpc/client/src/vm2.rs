// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use super::{map_err, RpcClient};
use starcoin_vm2_account_api::AccountInfo;
use starcoin_vm2_types::{
    account_address::AccountAddress,
    view::{SignedMessageView, StrView, TransactionRequest},
};
use starcoin_vm2_vm_types::{
    account_config::token_code::TokenCode,
    sign_message::SigningMessage,
    transaction::{RawUserTransaction, SignedUserTransaction},
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
}
