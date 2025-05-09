use crate::G_ENV_PRIVATE_KEY;
use anyhow::{bail, Result};
use starcoin_account::{account_storage::AccountStorage, AccountManager};
use starcoin_account_api::{AccountInfo, AccountPrivateKey, AccountProvider};
use starcoin_crypto::{ValidCryptoMaterial, ValidCryptoMaterialStringExt};
use starcoin_types::account_address::AccountAddress;
use starcoin_types::account_config::token_code::TokenCode;
use starcoin_types::genesis_config::ChainId;
use starcoin_types::sign_message::{SignedMessage, SigningMessage};
use starcoin_types::transaction::{RawUserTransaction, SignedUserTransaction};
use std::env;
use std::path::PathBuf;
use std::time::Duration;

pub struct AccountPrivateKeyProvider {
    manager: AccountManager,
}

impl AccountPrivateKeyProvider {
    pub fn create(
        secret_file: Option<PathBuf>,
        address: Option<AccountAddress>,
        from_env: bool,
        chain_id: ChainId,
    ) -> Result<Self> {
        if !(secret_file.is_some() ^ from_env) {
            bail!("Please input one and only one in args [secret_file, from_env].")
        }
        let storage = AccountStorage::mock();
        let manager = AccountManager::new(storage, chain_id)?;

        let data = if secret_file.is_some() {
            std::fs::read_to_string(secret_file.unwrap())?
        } else {
            match env::var_os(G_ENV_PRIVATE_KEY) {
                Some(value) => value.into_string().unwrap_or_else(|_| String::from("")),
                None => String::from(""),
            }
        };
        let data = data.trim_end_matches('\n').trim_end_matches('\r');
        if data.is_empty() {
            bail!("Invalid private key.")
        };

        let private_key = AccountPrivateKey::from_encoded_string(data)?;
        let address = address.unwrap_or_else(|| private_key.public_key().derived_address());
        let _account = manager.import_account(address, private_key.to_bytes().to_vec(), "")?;
        Ok(Self { manager })
    }
}
impl AccountProvider for AccountPrivateKeyProvider {
    fn create_account(&self, _password: String) -> anyhow::Result<AccountInfo> {
        bail!("Unsupported")
    }

    fn get_default_account(&self) -> anyhow::Result<Option<AccountInfo>> {
        self.manager.default_account_info().map_err(|e| e.into())
    }

    fn set_default_account(&self, _address: AccountAddress) -> anyhow::Result<AccountInfo> {
        bail!("Unsupported")
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
        _password: String,
        duration: Duration,
    ) -> anyhow::Result<AccountInfo> {
        self.manager
            .unlock_account(address, "", duration)
            .map_err(|e| e.into())
    }

    fn lock_account(&self, _address: AccountAddress) -> anyhow::Result<AccountInfo> {
        bail!("Unsupported")
    }

    fn import_account(
        &self,
        _address: AccountAddress,
        _private_key: Vec<u8>,
        _password: String,
    ) -> anyhow::Result<AccountInfo> {
        bail!("Unsupported")
    }

    fn import_readonly_account(
        &self,
        _address: AccountAddress,
        _public_key: Vec<u8>,
    ) -> anyhow::Result<AccountInfo> {
        bail!("Unsupported")
    }

    fn export_account(
        &self,
        _address: AccountAddress,
        _password: String,
    ) -> anyhow::Result<Vec<u8>> {
        bail!("Unsupported")
    }

    fn accepted_tokens(&self, address: AccountAddress) -> anyhow::Result<Vec<TokenCode>> {
        self.manager.accepted_tokens(address).map_err(|e| e.into())
    }

    fn change_account_password(
        &self,
        _address: AccountAddress,
        _new_password: String,
    ) -> anyhow::Result<AccountInfo> {
        bail!("Unsupported")
    }

    fn remove_account(
        &self,
        _address: AccountAddress,
        _password: Option<String>,
    ) -> anyhow::Result<AccountInfo> {
        bail!("Unsupported")
    }
}
