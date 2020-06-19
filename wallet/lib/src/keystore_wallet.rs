// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::format_err;
use rand::prelude::*;
use starcoin_crypto::ed25519::{Ed25519PrivateKey, Ed25519PublicKey};
use starcoin_crypto::Uniform;
use starcoin_decrypt::{decrypt, encrypt};
use starcoin_types::transaction::helpers::TransactionSigner;
use starcoin_types::{
    account_address::{self, AccountAddress},
    transaction::{RawUserTransaction, SignedUserTransaction},
};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::ops::Add;
use std::sync::{Mutex, RwLock};
use std::time::Duration;
use std::time::Instant;
use wallet_api::{error::WalletError, Wallet, WalletAccount, WalletStore};

type KeyPair = starcoin_crypto::test_utils::KeyPair<Ed25519PrivateKey, Ed25519PublicKey>;
pub type Result<T> = std::result::Result<T, WalletError>;

/// Wallet base KeyStore
/// encrypt account's key by a password.
#[derive(Default, Debug)]
pub struct KeyStoreWallet<TKeyStore> {
    store: TKeyStore,
    default_account: Mutex<Option<WalletAccount>>,
    key_cache: RwLock<KeyCache>,
}

#[derive(Default, Debug, PartialEq, Eq)]
struct KeyCache {
    cache: HashMap<AccountAddress, (Instant, KeyPair)>,
}
impl KeyCache {
    pub fn cache_key(&mut self, account: AccountAddress, keypair: KeyPair, ttl: Instant) {
        self.cache.insert(account, (ttl, keypair));
    }
    pub fn remove_key(&mut self, account: &AccountAddress) {
        self.cache.remove(account);
    }
    pub fn get_key(&mut self, account: &AccountAddress) -> Option<&KeyPair> {
        match self.cache.remove(account) {
            None => None,
            Some((ttl, kp)) => {
                if Instant::now() < ttl {
                    self.cache.insert(account.clone(), (ttl, kp));
                    self.cache.get(account).map(|t| &t.1)
                } else {
                    None
                }
            }
        }
    }

    #[allow(dead_code)]
    pub fn clean_expired(&mut self) {
        let cur_instant = Instant::now();
        self.cache.retain(|_account, (ttl, _)| &cur_instant < ttl);
    }
}

impl<TKeyStore> Wallet for KeyStoreWallet<TKeyStore>
where
    TKeyStore: WalletStore,
{
    fn create_account(&self, password: &str) -> Result<WalletAccount> {
        let keypair = gen_keypair();
        let address = account_address::from_public_key(&keypair.public_key);
        let existed_accounts = self.store.get_accounts()?;
        //first account is default.
        let is_default = existed_accounts.is_empty();
        let account = WalletAccount::new(address, keypair.public_key.clone(), is_default);
        self.save_account(account.clone(), keypair, password.to_string())?;
        Ok(account)
    }

    fn get_account(&self, address: &AccountAddress) -> Result<Option<WalletAccount>> {
        Ok(self.store.get_account(address)?)
    }

    fn import_account(
        &self,
        address: AccountAddress,
        private_key: Vec<u8>,
        password: &str,
    ) -> Result<WalletAccount> {
        if self.contains(&address)? {
            return Err(WalletError::AccountAlreadyExist(address));
        }
        let private_key = Ed25519PrivateKey::try_from(private_key.as_slice())
            .map_err(|_| WalletError::InvalidPrivateKey)?;
        let key_pair = KeyPair::from(private_key);
        let account = WalletAccount::new(address, key_pair.public_key.clone(), false);
        self.save_account(account.clone(), key_pair, password.to_string())?;
        Ok(account)
    }
    fn export_account(&self, address: &AccountAddress, password: &str) -> Result<Vec<u8>> {
        let keypair = self.unlock_prikey(address, password)?;
        Ok(keypair.private_key.to_bytes().to_vec())
    }

    fn contains(&self, address: &AccountAddress) -> Result<bool> {
        self.get_account(address).map(|w| w.is_some())
    }

    fn unlock_account(
        &self,
        address: AccountAddress,
        password: &str,
        duration: Duration,
    ) -> Result<()> {
        let keypair = self.unlock_prikey(&address, password)?;
        let address = account_address::from_public_key(&keypair.public_key);
        let ttl = std::time::Instant::now().add(duration);
        self.key_cache
            .write()
            .unwrap()
            .cache_key(address, keypair, ttl);
        Ok(())
    }

    fn lock_account(&self, address: AccountAddress) -> Result<()> {
        self.key_cache.write().unwrap().remove_key(&address);
        Ok(())
    }

    fn sign_txn(
        &self,
        raw_txn: RawUserTransaction,
        signer_address: AccountAddress,
    ) -> Result<SignedUserTransaction> {
        if !self.contains(&signer_address)? {
            return Err(WalletError::AccountNotExist(signer_address));
        }
        match self.key_cache.write().unwrap().get_key(&signer_address) {
            None => Err(WalletError::AccountLocked(signer_address)),
            Some(k) => k
                .sign_txn(raw_txn)
                .map_err(WalletError::TransactionSignError),
        }
    }

    fn get_default_account(&self) -> Result<Option<WalletAccount>> {
        let default_account = self.default_account.lock().unwrap().as_ref().cloned();
        match default_account {
            Some(a) => Ok(Some(a)),
            None => {
                let default_account = self
                    .store
                    .get_accounts()?
                    .into_iter()
                    .find(|account| account.is_default);
                *self.default_account.lock().unwrap() = default_account.clone();
                Ok(default_account)
            }
        }
    }

    fn get_accounts(&self) -> Result<Vec<WalletAccount>> {
        Ok(self.store.get_accounts()?)
    }

    fn set_default(&self, address: &AccountAddress) -> Result<()> {
        let mut target = self
            .get_account(address)?
            .ok_or_else(|| WalletError::AccountNotExist(*address))?;

        let default = self.get_default_account()?;
        if let Some(mut default) = default {
            if &default.address == address {
                return Ok(());
            }
            default.is_default = false;
            self.store.save_account(default)?;
        }

        target.is_default = true;
        self.store.save_account(target.clone())?;
        // save default into cache
        *self.default_account.lock().unwrap() = Some(target);
        Ok(())
    }

    fn remove_account(&self, address: &AccountAddress) -> Result<()> {
        if let Some(account) = self.get_account(address)? {
            if account.is_default {
                return Err(WalletError::RemoveDefaultAccountError(*address));
            }
            self.store.remove_account(address)?;
        }
        Ok(())
    }
}

fn gen_keypair() -> KeyPair {
    let mut seed_rng = rand::rngs::OsRng;
    let seed_buf: [u8; 32] = seed_rng.gen();
    let mut rng: StdRng = SeedableRng::from_seed(seed_buf);
    let key_pair: KeyPair = KeyPair::generate(&mut rng);
    key_pair
}

const KEY_NAME_ENCRYPTED_PRIVATE_KEY: &str = "encrypted_private_key";

impl<TKeyStore> KeyStoreWallet<TKeyStore>
where
    TKeyStore: WalletStore,
{
    pub fn new(keystore: TKeyStore) -> Result<Self> {
        let wallet = Self {
            store: keystore,
            default_account: Mutex::new(None),
            key_cache: RwLock::new(KeyCache::default()),
        };
        Ok(wallet)
    }

    fn save_account(
        &self,
        account: WalletAccount,
        key_pair: KeyPair,
        password: String,
    ) -> Result<()> {
        let address = account.address;
        self.store.save_account(account)?;
        let encrypted_prikey = encrypt(password.as_bytes(), &key_pair.private_key.to_bytes());
        self.store.save_to_account(
            &address,
            KEY_NAME_ENCRYPTED_PRIVATE_KEY.to_string(),
            encrypted_prikey,
        )?;
        Ok(())
    }

    fn unlock_prikey(&self, address: &AccountAddress, password: &str) -> Result<KeyPair> {
        let cached_public_key = {
            let mut cache_guard = self.key_cache.write().unwrap();
            cache_guard.get_key(address).map(|p| p.public_key.clone())
        };
        let account_public_key = match cached_public_key {
            Some(pub_key) => pub_key,
            None => match self.store.get_account(address)? {
                None => {
                    return Err(WalletError::AccountNotExist(*address));
                }
                Some(account) => account.public_key,
            },
        };

        let key_data = self
            .store
            .get_from_account(address, KEY_NAME_ENCRYPTED_PRIVATE_KEY)?;
        if key_data.is_none() {
            return Err(WalletError::AccountPrivateKeyMissing(*address));
        }

        let key_data = key_data.unwrap();
        let plain_key_data = decrypt(password.as_bytes(), &key_data)
            .map_err(|_e| WalletError::InvalidPassword(*address))?;
        let private_key = Ed25519PrivateKey::try_from(plain_key_data.as_slice()).map_err(|_e| {
            WalletError::StoreError(format_err!("underline vault store corrupted"))
        })?;
        let keypair = KeyPair::from(private_key);

        // check the private key does correspond the declared public key
        if keypair.public_key.to_bytes() != account_public_key.to_bytes() {
            return Err(WalletError::InvalidPassword(*address));
        }
        Ok(keypair)
    }
}

#[cfg(test)]
mod tests {
    use super::KeyStoreWallet;
    use super::RawUserTransaction;
    use super::Wallet;
    use crate::file_wallet_store::FileWalletStore;
    use crate::keystore_wallet::gen_keypair;
    use anyhow::Result;
    use starcoin_types::account_address;
    use std::time::Duration;

    #[test]
    fn test_wallet() -> Result<()> {
        let tmp_path = tempfile::tempdir()?;
        let wallet_store = FileWalletStore::new(tmp_path.path());
        let wallet = KeyStoreWallet::new(wallet_store)?;
        let account = wallet.get_default_account()?;
        assert!(account.is_none());
        let account = wallet.create_account("pass")?;
        assert!(account.is_default);
        wallet.unlock_account(account.address, "pass", Duration::from_secs(5))?;
        let raw_txn = RawUserTransaction::mock_by_sender(account.address);
        let signer_address = raw_txn.sender();
        let _txn = wallet.sign_txn(raw_txn, signer_address)?;
        wallet.lock_account(account.address)?;

        let account2 = wallet.create_account("pass2")?;
        assert!(!account2.is_default);
        wallet.set_default(&account2.address)?;
        let default_account = wallet.get_default_account()?;
        assert!(default_account.is_some());
        assert_eq!(account2.address, default_account.unwrap().address);

        Ok(())
    }
    #[test]
    fn test_wallet_import_account_and_sign() -> Result<()> {
        let tmp_path = tempfile::tempdir()?;
        let wallet_store = FileWalletStore::new(tmp_path.path());
        let wallet = KeyStoreWallet::new(wallet_store)?;
        let keypair = gen_keypair();
        let address = account_address::from_public_key(&keypair.public_key);
        let account =
            wallet.import_account(address, keypair.private_key.to_bytes().to_vec(), "pass")?;
        wallet.unlock_account(account.address, "pass", Duration::from_secs(10))?;
        let raw_txn = RawUserTransaction::mock_by_sender(account.address);
        let signed_txn = raw_txn
            .clone()
            .sign(&keypair.private_key, keypair.public_key.clone())?
            .into_inner();
        let signer_address = raw_txn.sender();
        let txn = wallet.sign_txn(raw_txn, signer_address)?;
        assert_eq!(signed_txn, txn);
        Ok(())
    }

    #[test]
    fn test_wallet_get_account_details() -> Result<()> {
        let tmp_path = tempfile::tempdir()?;
        let wallet_store = FileWalletStore::new(tmp_path.path());
        let wallet = KeyStoreWallet::new(wallet_store)?;
        let account = wallet.create_account("hello")?;
        let wallet_account = wallet.get_account(&account.address)?;
        assert!(wallet_account.is_some());
        let account_detail = wallet_account.unwrap();
        let address = account_address::from_public_key(&account_detail.public_key);
        assert_eq!(&address, account_detail.address());
        Ok(())
    }
}
