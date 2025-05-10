// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_rpc_client::RpcClient;
use starcoin_vm2_account_api::{AccountInfo, AccountProvider};
use starcoin_vm2_types::{
    account_address::AccountAddress,
    account_config::token_code::TokenCode,
    sign_message::{SignedMessage, SigningMessage},
    transaction::{RawUserTransaction, SignedUserTransaction},
};
use std::{sync::Arc, time::Duration};

pub struct AccountRpcProvider {
    rpc: Arc<RpcClient>,
}

impl AccountRpcProvider {
    pub fn create(rpc: Arc<RpcClient>) -> Self {
        Self { rpc }
    }
}

impl AccountProvider for AccountRpcProvider {
    fn create_account(&self, password: String) -> anyhow::Result<AccountInfo> {
        self.rpc.account_create2(password)
    }

    fn get_default_account(&self) -> anyhow::Result<Option<AccountInfo>> {
        self.rpc.account_default2()
    }

    fn set_default_account(&self, address: AccountAddress) -> anyhow::Result<AccountInfo> {
        self.rpc.set_default_account2(address)
    }

    fn get_accounts(&self) -> anyhow::Result<Vec<AccountInfo>> {
        self.rpc.account_list2()
    }

    fn get_account(&self, address: AccountAddress) -> anyhow::Result<Option<AccountInfo>> {
        self.rpc.account_get2(address)
    }

    fn sign_message(
        &self,
        address: AccountAddress,
        message: SigningMessage,
    ) -> anyhow::Result<SignedMessage> {
        let signed_message = self.rpc.account_sign_message2(address, message)?;
        Ok(signed_message.0)
    }

    fn sign_txn(
        &self,
        raw_txn: RawUserTransaction,
        _signer_address: AccountAddress,
    ) -> anyhow::Result<SignedUserTransaction> {
        self.rpc.account_sign_txn2(raw_txn)
    }

    fn unlock_account(
        &self,
        address: AccountAddress,
        password: String,
        duration: Duration,
    ) -> anyhow::Result<AccountInfo> {
        self.rpc.account_unlock2(address, password, duration)
    }

    fn lock_account(&self, address: AccountAddress) -> anyhow::Result<AccountInfo> {
        self.rpc.account_lock2(address)
    }

    fn import_account(
        &self,
        address: AccountAddress,
        private_key: Vec<u8>,
        password: String,
    ) -> anyhow::Result<AccountInfo> {
        self.rpc.account_import2(address, private_key, password)
    }

    fn import_readonly_account(
        &self,
        address: AccountAddress,
        public_key: Vec<u8>,
    ) -> anyhow::Result<AccountInfo> {
        self.rpc.account_import_readonly2(address, public_key)
    }

    fn export_account(&self, address: AccountAddress, password: String) -> anyhow::Result<Vec<u8>> {
        self.rpc.account_export2(address, password)
    }

    fn accepted_tokens(&self, address: AccountAddress) -> anyhow::Result<Vec<TokenCode>> {
        self.rpc.account_accepted_tokens2(address)
    }

    fn change_account_password(
        &self,
        address: AccountAddress,
        new_password: String,
    ) -> anyhow::Result<AccountInfo> {
        self.rpc.account_change_password2(address, new_password)
    }

    fn remove_account(
        &self,
        address: AccountAddress,
        password: Option<String>,
    ) -> anyhow::Result<AccountInfo> {
        self.rpc.account_remove2(address, password)
    }
}
