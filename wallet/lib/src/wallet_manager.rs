// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::wallet::Wallet;
use crate::wallet_storage::WalletStorage;

use parking_lot::RwLock;
use rand::prelude::*;
use starcoin_crypto::ed25519::{Ed25519PrivateKey, Ed25519PublicKey};
use starcoin_crypto::{PrivateKey, Uniform};
use starcoin_types::account_config::token_code::TokenCode;
use starcoin_types::{
    account_address::{self, AccountAddress},
    transaction::{RawUserTransaction, SignedUserTransaction},
};
use starcoin_wallet_api::error::WalletError;
use starcoin_wallet_api::WalletAccount;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::ops::Add;
use std::time::Duration;
use std::time::Instant;

type KeyPair = starcoin_crypto::test_utils::KeyPair<Ed25519PrivateKey, Ed25519PublicKey>;
pub type Result<T> = std::result::Result<T, WalletError>;

/// Wallet base KeyStore
/// encrypt account's key by a password.
pub struct WalletManager {
    store: WalletStorage,
    key_cache: RwLock<PasswordCache>,
}

#[derive(Default, Debug, PartialEq, Eq)]
struct PasswordCache {
    cache: HashMap<AccountAddress, (Instant, String)>,
}
impl PasswordCache {
    pub fn cache_pass(&mut self, account: AccountAddress, pass: String, ttl: Instant) {
        self.cache.insert(account, (ttl, pass));
    }
    pub fn remove_pass(&mut self, account: &AccountAddress) {
        self.cache.remove(account);
    }
    pub fn get_pass(&mut self, account: &AccountAddress) -> Option<String> {
        match self.cache.remove(account) {
            None => None,
            Some((ttl, kp)) => {
                if Instant::now() < ttl {
                    self.cache.insert(*account, (ttl, kp));
                    self.cache.get(account).map(|t| t.1.to_string())
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

impl WalletManager {
    pub fn new(storage: WalletStorage) -> Result<Self> {
        let manager = Self {
            store: storage,
            key_cache: RwLock::new(PasswordCache::default()),
        };
        Ok(manager)
    }

    pub fn create_wallet(&self, password: &str) -> Result<Wallet> {
        let keypair = gen_keypair();
        let address = account_address::from_public_key(&keypair.public_key);

        let wallet = Wallet::create(
            keypair.public_key.clone(),
            keypair.private_key,
            Some(address),
            password.to_string(),
            self.store.clone(),
        )?;
        self.store.add_address(*wallet.address())?;

        // if it's the first address, set it default.
        if self.store.list_addresses()?.len() == 1 {
            self.set_default_wallet(address)?;
        }

        Ok(wallet)
    }

    pub fn unlock_wallet(
        &self,
        address: AccountAddress,
        password: &str,
        duration: Duration,
    ) -> Result<()> {
        let _ = Wallet::load(address, password, self.store.clone())?
            .ok_or_else(|| WalletError::AccountNotExist(address))?;
        let ttl = std::time::Instant::now().add(duration);
        self.key_cache
            .write()
            .cache_pass(address, password.to_string(), ttl);
        Ok(())
    }

    pub fn lock_wallet(&self, address: AccountAddress) -> Result<()> {
        self.key_cache.write().remove_pass(&address);
        Ok(())
    }

    pub fn import_wallet(
        &self,
        address: AccountAddress,
        private_key: Vec<u8>,
        password: &str,
    ) -> Result<Wallet> {
        if self.contains(&address)? {
            return Err(WalletError::AccountAlreadyExist(address));
        }
        let private_key = Ed25519PrivateKey::try_from(private_key.as_slice())
            .map_err(|_| WalletError::InvalidPrivateKey)?;
        // let key_pair = KeyPair::from(private_key);
        let wallet = Wallet::create(
            private_key.public_key(),
            private_key,
            Some(address),
            password.to_string(),
            self.store.clone(),
        )?;
        Ok(wallet)
    }

    pub fn export_wallet(&self, address: AccountAddress, password: &str) -> Result<Vec<u8>> {
        let wallet = Wallet::load(address, password, self.store.clone())?
            .ok_or_else(|| WalletError::AccountNotExist(address))?;
        Ok(wallet.private_key().to_bytes().to_vec())
    }

    pub fn contains(&self, address: &AccountAddress) -> Result<bool> {
        self.store
            .contain_address(*address)
            .map_err(WalletError::StoreError)
    }

    pub fn default_wallet_info(&self) -> Result<Option<WalletAccount>> {
        let default_address = self.store.default_address()?;
        let wallet_info = default_address
            .map(|a| self.wallet_info(a))
            .transpose()?
            .and_then(|a| a);
        Ok(wallet_info)
    }
    pub fn list_wallet_infos(&self) -> Result<Vec<WalletAccount>> {
        let default_account = self.store.default_address()?;
        let mut res = vec![];
        for account in self.store.list_addresses()? {
            let pubkey = self.store.public_key(account)?;
            match pubkey {
                Some(p) => {
                    res.push(WalletAccount {
                        address: account,
                        is_default: default_account.filter(|a| a == &account).is_some(),
                        public_key: p,
                    });
                }
                None => {
                    continue;
                }
            }
        }
        Ok(res)
    }

    pub fn wallet_info(&self, address: AccountAddress) -> Result<Option<WalletAccount>> {
        match self.store.public_key(address)? {
            Some(p) => {
                let default_account = self.store.default_address()?;
                Ok(Some(WalletAccount {
                    address,
                    is_default: default_account.filter(|a| a == &address).is_some(),
                    public_key: p,
                }))
            }
            None => Ok(None),
        }
    }

    pub fn sign_txn(
        &self,
        signer_address: AccountAddress,
        raw_txn: RawUserTransaction,
    ) -> Result<SignedUserTransaction> {
        let pass = self.key_cache.write().get_pass(&signer_address);
        match pass {
            None => Err(WalletError::AccountLocked(signer_address)),
            Some(p) => {
                let wallet = Wallet::load(signer_address, p.as_str(), self.store.clone())?
                    .ok_or_else(|| WalletError::AccountNotExist(signer_address))?;
                wallet
                    .sign_txn(raw_txn)
                    .map_err(WalletError::TransactionSignError)
            }
        }
    }

    #[allow(unused)]
    pub fn set_default_wallet(&self, address: AccountAddress) -> Result<()> {
        self.store
            .set_default_address(Some(address))
            .map_err(WalletError::StoreError)
    }

    /// remove wallet need user password.
    #[allow(unused)]
    pub fn delete_wallet(&self, address: AccountAddress, password: &str) -> Result<()> {
        let wallet = Wallet::load(address, password, self.store.clone())?;
        match wallet {
            Some(wallet) => {
                self.key_cache.write().remove_pass(&address);
                wallet.destroy().map_err(WalletError::StoreError)
            }
            None => Ok(()),
        }
    }

    pub fn accepted_tokens(&self, address: AccountAddress) -> Result<Vec<TokenCode>> {
        self.store
            .get_accepted_tokens(address)
            .map_err(WalletError::StoreError)
    }
}

fn gen_keypair() -> KeyPair {
    let mut seed_rng = rand::rngs::OsRng;
    let seed_buf: [u8; 32] = seed_rng.gen();
    let mut rng: StdRng = SeedableRng::from_seed(seed_buf);
    let key_pair: KeyPair = KeyPair::generate(&mut rng);
    key_pair
}
