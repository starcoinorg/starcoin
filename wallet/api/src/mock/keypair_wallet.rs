// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::error::WalletError;
use crate::mock::MemWalletStore;
use crate::{Wallet, WalletAccount, WalletResult, WalletStore};
use anyhow::{format_err, Result};
use rand::prelude::*;
use starcoin_crypto::ed25519::{Ed25519PrivateKey, Ed25519PublicKey};
use starcoin_crypto::Uniform;
use starcoin_types::transaction::helpers::TransactionSigner;
use starcoin_types::{
    account_address::AccountAddress,
    transaction::{RawUserTransaction, SignedUserTransaction},
};
use std::convert::TryFrom;
use std::time::Duration;

type KeyPair = starcoin_crypto::test_utils::KeyPair<Ed25519PrivateKey, Ed25519PublicKey>;

/// Save raw key, ignore password, just for test.
pub struct KeyPairWallet<S>
where
    S: WalletStore,
{
    store: S,
}

impl KeyPairWallet<MemWalletStore> {
    pub fn new() -> Result<Self> {
        Self::new_with_store(MemWalletStore::new())
    }
}

impl<S> KeyPairWallet<S>
where
    S: WalletStore,
{
    pub fn new_with_store(store: S) -> Result<Self> {
        let wallet = Self { store };
        if wallet.get_accounts()?.is_empty() {
            wallet.create_account("")?;
        }
        Ok(wallet)
    }

    fn save_account(&self, account: WalletAccount, key_pair: KeyPair) -> WalletResult<()> {
        let address = account.address;
        self.store.save_account(account)?;
        self.store.save_to_account(
            &address,
            KEY_NAME_PRIVATE_KEY.to_string(),
            key_pair.private_key.to_bytes().to_vec(),
        )?;
        Ok(())
    }

    fn get_key_pair(&self, address: &AccountAddress) -> WalletResult<KeyPair> {
        let private_key = self.store.get_from_account(address, KEY_NAME_PRIVATE_KEY)?;
        if private_key.is_none() {
            return Err(WalletError::StoreError(format_err!(
                "canot find private key by address: {}",
                address
            )));
        }
        let private_key = private_key.unwrap();
        let private_key = Ed25519PrivateKey::try_from(private_key.as_slice()).map_err(|_| {
            WalletError::StoreError(format_err!(
                "cannot decode private key from underline bytes"
            ))
        })?;
        Ok(KeyPair::from(private_key))
    }
}

const KEY_NAME_PRIVATE_KEY: &str = "private_key";

impl<S> Wallet for KeyPairWallet<S>
where
    S: WalletStore,
{
    fn create_account(&self, _password: &str) -> WalletResult<WalletAccount> {
        let key_pair: KeyPair = KeyPair::generate_for_testing();
        let address = AccountAddress::from_public_key(&key_pair.public_key);
        //first account is default.
        let is_default = self.get_accounts()?.len() == 0;
        let account = WalletAccount::new(address, key_pair.public_key.clone(), is_default);
        self.save_account(account.clone(), key_pair)?;
        Ok(account)
    }

    fn get_account(&self, address: &AccountAddress) -> WalletResult<Option<WalletAccount>> {
        Ok(self.store.get_account(address)?)
    }

    fn import_account(
        &self,
        address: AccountAddress,
        private_key: Vec<u8>,
        _password: &str,
    ) -> WalletResult<WalletAccount> {
        let private_key = Ed25519PrivateKey::try_from(private_key.as_slice())
            .map_err(|_e| WalletError::InvalidPrivateKey)?;
        let key_pair = KeyPair::from(private_key);
        let account = WalletAccount::new(address, key_pair.public_key.clone(), false);
        self.save_account(account.clone(), key_pair)?;
        Ok(account)
    }
    fn export_account(&self, address: &AccountAddress, _password: &str) -> WalletResult<Vec<u8>> {
        self.get_key_pair(address)
            .map(|kp| kp.private_key.to_bytes().to_vec())
    }

    fn contains(&self, address: &AccountAddress) -> WalletResult<bool> {
        Ok(self.store.get_account(address)?.map(|_| true).is_some())
    }

    fn unlock_account(
        &self,
        _address: AccountAddress,
        _password: &str,
        _duration: Duration,
    ) -> WalletResult<()> {
        //do nothing
        Ok(())
    }

    fn lock_account(&self, _address: AccountAddress) -> WalletResult<()> {
        //do nothing
        Ok(())
    }

    fn sign_txn(&self, raw_txn: RawUserTransaction) -> WalletResult<SignedUserTransaction> {
        let address = raw_txn.sender();
        if !self.contains(&address)? {
            return Err(WalletError::AccountNotExist(address.clone()));
        }
        let key_pair = self.get_key_pair(&address)?;
        key_pair
            .sign_txn(raw_txn)
            .map_err(|e| WalletError::TransactionSignError(e))
    }

    fn get_default_account(&self) -> WalletResult<Option<WalletAccount>> {
        Ok(self
            .store
            .get_accounts()?
            .iter()
            .find(|account| account.is_default)
            .cloned())
    }

    fn get_accounts(&self) -> WalletResult<Vec<WalletAccount>> {
        Ok(self.store.get_accounts()?)
    }

    fn set_default(&self, address: &AccountAddress) -> WalletResult<()> {
        let mut target = self
            .get_account(address)?
            .ok_or(WalletError::AccountNotExist(address.clone()))?;

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

        Ok(())
    }

    fn remove_account(&self, address: &AccountAddress) -> WalletResult<()> {
        let account = self.get_account(address)?;
        match account {
            Some(account) => {
                if account.is_default {
                    return Err(WalletError::RemoveDefaultAccountError(address.clone()));
                }
                self.store.remove_account(address)?;
            }
            None => {}
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet() -> Result<()> {
        let wallet = KeyPairWallet::new()?;
        let account = wallet.get_default_account()?;
        assert!(account.is_some());
        let account = account.unwrap();
        let raw_txn = RawUserTransaction::mock_by_sender(account.address);
        let _txn = wallet.sign_txn(raw_txn)?;
        Ok(())
    }
}
