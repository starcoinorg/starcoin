// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, ensure, format_err, Error, Result};
use rand::prelude::*;
use starcoin_crypto::ed25519::{Ed25519PrivateKey, Ed25519PublicKey};
use starcoin_crypto::Uniform;
use starcoin_decrypt::{decrypt, encrypt};
use starcoin_types::transaction::helpers::TransactionSigner;
use starcoin_types::{
    account_address::AccountAddress,
    transaction::{RawUserTransaction, SignedUserTransaction},
};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::ops::Add;
use std::sync::{Mutex, RwLock};
use std::time::Duration;
use std::time::Instant;
use wallet_api::{AccountWithKey, Wallet, WalletAccount, WalletStore};

type KeyPair = starcoin_crypto::test_utils::KeyPair<Ed25519PrivateKey, Ed25519PublicKey>;

/// Wallet base KeyStore
/// encrypt account's key by a password.
#[derive(Default, Debug)]
pub struct KeyStoreWallet<TKeyStore> {
    store: TKeyStore,
    default_account: Mutex<Option<AccountAddress>>,
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
                if &Instant::now() < &ttl {
                    self.cache.insert(account.clone(), (ttl, kp));
                    return self.cache.get(account).map(|t| &t.1);
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
    fn create_account(&self, password: &str) -> Result<WalletAccount, Error> {
        let keypair = gen_keypair();
        let address = AccountAddress::from_public_key(&keypair.public_key);
        let existed_accounts = self.store.get_accounts()?;
        //first account is default.
        let is_default = existed_accounts.len() == 0;
        let account = WalletAccount::new(address, is_default);
        self.save_account(account.clone(), keypair, password.to_string())?;
        Ok(account)
    }

    fn get_account_with_key(&self, _address: &AccountAddress) -> Result<Option<AccountWithKey>> {
        //TODO
        unimplemented!()
    }

    fn get_account(&self, address: &AccountAddress) -> Result<Option<WalletAccount>, Error> {
        self.store.get_account(address)
    }

    fn import_account(&self, private_key: Vec<u8>, password: &str) -> Result<WalletAccount, Error> {
        let private_key = Ed25519PrivateKey::try_from(private_key.as_slice())?;
        let key_pair = KeyPair::from(private_key);
        let address = AccountAddress::from_public_key(&key_pair.public_key);
        let account = WalletAccount::new(address, false);
        self.save_account(account.clone(), key_pair, password.to_string())?;
        Ok(account)
    }

    fn contains(&self, address: &AccountAddress) -> Result<bool, Error> {
        self.get_account(address).map(|w| w.is_some())
    }

    fn unlock_account(
        &self,
        address: AccountAddress,
        password: &str,
        duration: Duration,
    ) -> Result<(), Error> {
        let key_data = self
            .store
            .get_from_account(&address, KEY_NAME_ENCRYPTED_PRIVATE_KEY)?;
        ensure!(
            key_data.is_some(),
            "no private key data associate with address {}",
            &address
        );
        let key_data = key_data.unwrap();
        let plain_key_data = decrypt(password.as_bytes(), &key_data)?;
        let private_key = Ed25519PrivateKey::try_from(plain_key_data.as_slice())?;
        let keypair = KeyPair::from(private_key);
        let address = AccountAddress::from_public_key(&keypair.public_key);
        let ttl = std::time::Instant::now().add(duration);
        self.key_cache
            .write()
            .unwrap()
            .cache_key(address, keypair, ttl);
        Ok(())
    }

    fn lock_account(&self, address: AccountAddress) -> Result<(), Error> {
        self.key_cache.write().unwrap().remove_key(&address);
        Ok(())
    }

    fn sign_txn(&self, raw_txn: RawUserTransaction) -> Result<SignedUserTransaction, Error> {
        let address = raw_txn.sender();
        ensure!(
            self.contains(&address)?,
            "Can not find account by address: {:?}",
            address
        );
        match self.key_cache.write().unwrap().get_key(&address) {
            None => {
                bail!("account {} is locked, please unlock it", address);
            }
            Some(k) => k.sign_txn(raw_txn),
        }
    }

    fn get_default_account(&self) -> Result<Option<WalletAccount>, Error> {
        let default_address = self.default_account.lock().unwrap().as_ref().cloned();
        match default_address {
            Some(a) => Ok(Some(WalletAccount::new(a, true))),
            None => {
                let default_account = self
                    .store
                    .get_accounts()?
                    .into_iter()
                    .find(|account| account.is_default);
                *self.default_account.lock().unwrap() =
                    default_account.as_ref().map(|account| account.address);
                Ok(default_account)
            }
        }
    }

    fn get_accounts(&self) -> Result<Vec<WalletAccount>, Error> {
        self.store.get_accounts()
    }

    fn set_default(&self, address: &AccountAddress) -> Result<(), Error> {
        let mut target = self.get_account(address)?.ok_or(format_err!(
            "Can not find account by address: {:?}",
            address
        ))?;

        let default = self.get_default_account()?;
        if let Some(mut default) = default {
            if &default.address == address {
                return Ok(());
            }
            default.is_default = false;
            self.store.save_account(default)?;
        }

        target.is_default = true;
        self.store.save_account(target)?;
        // save default into cache
        *self.default_account.lock().unwrap() = Some(address.clone());
        Ok(())
    }

    fn remove_account(&self, address: &AccountAddress) -> Result<(), Error> {
        if let Some(account) = self.get_account(address)? {
            ensure!(!account.is_default, "Can not remove default account.");
            self.store.remove_account(address)?;
        }
        Ok(())
    }
}

fn gen_keypair() -> KeyPair {
    let mut seed_rng = rand::rngs::OsRng::new().expect("can't access OsRng");
    let seed_buf: [u8; 32] = seed_rng.gen();
    let mut rng: StdRng = SeedableRng::from_seed(seed_buf);
    let key_pair: KeyPair = KeyPair::generate_for_testing(&mut rng);
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file_wallet_store::FileWalletStore;
    // use wallet_api::mock::MemWalletStore;

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
        let _txn = wallet.sign_txn(raw_txn)?;
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

        let account = wallet.import_account(keypair.private_key.to_bytes().to_vec(), "pass")?;
        wallet.unlock_account(account.address, "pass", Duration::from_secs(10))?;
        let raw_txn = RawUserTransaction::mock_by_sender(account.address);
        let signed_txn = raw_txn
            .clone()
            .sign(&keypair.private_key, keypair.public_key.clone())?
            .into_inner();
        let txn = wallet.sign_txn(raw_txn)?;
        assert_eq!(signed_txn, txn);
        Ok(())
    }
}
