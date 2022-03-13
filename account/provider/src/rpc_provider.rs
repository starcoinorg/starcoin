use starcoin_account_api::AccountInfo;
use starcoin_account_api::AccountProvider;
use starcoin_rpc_client::RpcClient;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::account_config::token_code::TokenCode;
use starcoin_types::sign_message::{SignedMessage, SigningMessage};

use starcoin_types::transaction::{RawUserTransaction, SignedUserTransaction};
use std::sync::Arc;
use std::time::Duration;

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
        self.rpc.account_create(password)
    }

    fn get_default_account(&self) -> anyhow::Result<Option<AccountInfo>> {
        self.rpc.account_default()
    }

    fn set_default_account(&self, address: AccountAddress) -> anyhow::Result<AccountInfo> {
        self.rpc.set_default_account(address)
    }

    fn get_accounts(&self) -> anyhow::Result<Vec<AccountInfo>> {
        self.rpc.account_list()
    }

    fn get_account(&self, address: AccountAddress) -> anyhow::Result<Option<AccountInfo>> {
        self.rpc.account_get(address)
    }

    fn sign_message(
        &self,
        address: AccountAddress,
        message: SigningMessage,
    ) -> anyhow::Result<SignedMessage> {
        let signed_message = self.rpc.account_sign_message(address, message)?;
        Ok(signed_message.0)
    }

    fn sign_txn(
        &self,
        raw_txn: RawUserTransaction,
        _signer_address: AccountAddress,
    ) -> anyhow::Result<SignedUserTransaction> {
        self.rpc.account_sign_txn(raw_txn)
    }

    fn unlock_account(
        &self,
        address: AccountAddress,
        password: String,
        duration: Duration,
    ) -> anyhow::Result<AccountInfo> {
        self.rpc.account_unlock(address, password, duration)
    }

    fn lock_account(&self, address: AccountAddress) -> anyhow::Result<AccountInfo> {
        self.rpc.account_lock(address)
    }

    fn import_account(
        &self,
        address: AccountAddress,
        private_key: Vec<u8>,
        password: String,
    ) -> anyhow::Result<AccountInfo> {
        self.rpc.account_import(address, private_key, password)
    }

    fn import_readonly_account(
        &self,
        address: AccountAddress,
        public_key: Vec<u8>,
    ) -> anyhow::Result<AccountInfo> {
        self.rpc.account_import_readonly(address, public_key)
    }

    fn export_account(&self, address: AccountAddress, password: String) -> anyhow::Result<Vec<u8>> {
        self.rpc.account_export(address, password)
    }

    fn accepted_tokens(&self, address: AccountAddress) -> anyhow::Result<Vec<TokenCode>> {
        self.rpc.account_accepted_tokens(address)
    }

    fn change_account_password(
        &self,
        address: AccountAddress,
        new_password: String,
    ) -> anyhow::Result<AccountInfo> {
        self.rpc.account_change_password(address, new_password)
    }

    fn remove_account(
        &self,
        address: AccountAddress,
        password: Option<String>,
    ) -> anyhow::Result<AccountInfo> {
        self.rpc.account_remove(address, password)
    }
}
