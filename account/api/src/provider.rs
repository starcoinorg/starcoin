use crate::AccountInfo;
use anyhow::Result;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::account_config::token_code::TokenCode;
use starcoin_types::sign_message::{SignedMessage, SigningMessage};
use starcoin_types::transaction::{RawUserTransaction, SignedUserTransaction};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AccountProviderStrategy {
    RPC,
    Local,
}

impl Default for AccountProviderStrategy {
    fn default() -> Self {
        AccountProviderStrategy::RPC
    }
}
pub trait AccountProvider {
    fn create_account(&self, password: String) -> Result<AccountInfo>;

    fn get_default_account(&self) -> Result<Option<AccountInfo>>;
    fn set_default_account(&self, address: AccountAddress) -> Result<AccountInfo>;
    fn get_accounts(&self) -> Result<Vec<AccountInfo>>;

    fn get_account(&self, address: AccountAddress) -> Result<Option<AccountInfo>>;

    /// Signs the hash of data with given address.
    fn sign_message(
        &self,
        address: AccountAddress,
        message: SigningMessage,
    ) -> Result<SignedMessage>;

    fn sign_txn(
        &self,
        raw_txn: RawUserTransaction,
        signer_address: AccountAddress,
    ) -> Result<SignedUserTransaction>;
    fn unlock_account(
        &self,
        address: AccountAddress,
        password: String,
        duration: std::time::Duration,
    ) -> Result<AccountInfo>;
    fn lock_account(&self, address: AccountAddress) -> Result<AccountInfo>;
    fn import_account(
        &self,
        address: AccountAddress,
        private_key: Vec<u8>,
        password: String,
    ) -> Result<AccountInfo>;

    fn import_readonly_account(
        &self,
        address: AccountAddress,
        public_key: Vec<u8>,
    ) -> Result<AccountInfo>;

    /// Return the private key as bytes for `address`
    fn export_account(&self, address: AccountAddress, password: String) -> Result<Vec<u8>>;

    fn accepted_tokens(&self, address: AccountAddress) -> Result<Vec<TokenCode>>;

    /// change account password, user need to unlock account first.
    fn change_account_password(
        &self,
        address: AccountAddress,
        new_password: String,
    ) -> Result<AccountInfo>;

    fn remove_account(
        &self,
        address: AccountAddress,
        password: Option<String>,
    ) -> Result<AccountInfo>;
}
