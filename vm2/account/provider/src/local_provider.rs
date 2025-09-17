use anyhow::Result;
use starcoin_account::{account_storage::AccountStorage, AccountManager};
use starcoin_account_api::{AccountInfo, AccountProvider};
use starcoin_config::RocksdbConfig;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::account_config::token_code::TokenCode;
use starcoin_types::genesis_config::ChainId;
use starcoin_types::sign_message::{SignedMessage, SigningMessage};
use starcoin_types::transaction::{RawUserTransaction, SignedUserTransaction};
use std::path::Path;
use std::time::Duration;
pub struct AccountLocalProvider {
    manager: AccountManager,
}

impl AccountLocalProvider {
    pub fn create(path: &Path, chain_id: ChainId) -> Result<Self> {
        let storage = AccountStorage::create_from_path(path, RocksdbConfig::default())?;
        let manager = AccountManager::new(storage, chain_id)?;
        Ok(Self { manager })
    }
}

impl AccountProvider for AccountLocalProvider {
    fn create_account(&self, password: String) -> anyhow::Result<AccountInfo> {
        let account_info = self
            .manager
            .create_account(password.as_str())
            .map(|account| account.info())?;
        Ok(account_info)
    }

    fn get_default_account(&self) -> anyhow::Result<Option<AccountInfo>> {
        self.manager.default_account_info().map_err(|e| e.into())
    }

    fn set_default_account(&self, address: AccountAddress) -> anyhow::Result<AccountInfo> {
        self.manager
            .set_default_account(address)
            .map_err(|e| e.into())
    }

    fn get_accounts(&self) -> anyhow::Result<Vec<AccountInfo>> {
        self.manager.list_account_infos().map_err(|e| e.into())
    }

    fn get_account(&self, address: AccountAddress) -> anyhow::Result<Option<AccountInfo>> {
        self.manager.account_info(address).map_err(|e| e.into())
    }

    fn sign_message(
        &self,
        address: AccountAddress,
        message: SigningMessage,
    ) -> anyhow::Result<SignedMessage> {
        self.manager
            .sign_message(address, message)
            .map_err(|e| e.into())
    }

    fn sign_txn(
        &self,
        raw_txn: RawUserTransaction,
        signer_address: AccountAddress,
    ) -> anyhow::Result<SignedUserTransaction> {
        self.manager
            .sign_txn(signer_address, raw_txn)
            .map_err(|e| e.into())
    }

    fn unlock_account(
        &self,
        address: AccountAddress,
        password: String,
        duration: Duration,
    ) -> anyhow::Result<AccountInfo> {
        self.manager
            .unlock_account(address, password.as_str(), duration)
            .map_err(|e| e.into())
    }

    fn lock_account(&self, address: AccountAddress) -> anyhow::Result<AccountInfo> {
        self.manager.lock_account(address).map_err(|e| e.into())
    }

    fn import_account(
        &self,
        address: AccountAddress,
        private_key: Vec<u8>,
        password: String,
    ) -> anyhow::Result<AccountInfo> {
        self.manager
            .import_account(address, private_key, password.as_str())
            .map_err(|e| e.into())
            .map(|account| account.info())
    }

    fn import_readonly_account(
        &self,
        address: AccountAddress,
        public_key: Vec<u8>,
    ) -> anyhow::Result<AccountInfo> {
        self.manager
            .import_readonly_account(address, public_key)
            .map_err(|e| e.into())
            .map(|account| account.info())
    }

    fn export_account(&self, address: AccountAddress, password: String) -> anyhow::Result<Vec<u8>> {
        self.manager
            .export_account(address, password.as_str())
            .map_err(|e| e.into())
    }

    fn accepted_tokens(&self, address: AccountAddress) -> anyhow::Result<Vec<TokenCode>> {
        self.manager.accepted_tokens(address).map_err(|e| e.into())
    }

    fn change_account_password(
        &self,
        address: AccountAddress,
        new_password: String,
    ) -> anyhow::Result<AccountInfo> {
        self.manager
            .change_password(address, new_password)
            .map_err(|e| e.into())
    }

    fn remove_account(
        &self,
        address: AccountAddress,
        password: Option<String>,
    ) -> anyhow::Result<AccountInfo> {
        self.manager
            .remove_account(address, password)
            .map_err(|e| e.into())
    }
}
