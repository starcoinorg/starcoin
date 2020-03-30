// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::mock::MemWalletStore;
use crate::{AccountDetail, Wallet, WalletAccount, WalletStore};
use actix::clock::Duration;
use anyhow::{ensure, format_err, Result};
use rand::prelude::*;
use starcoin_crypto::ed25519::{Ed25519PrivateKey, Ed25519PublicKey};
use starcoin_crypto::Uniform;
use starcoin_types::transaction::helpers::TransactionSigner;
use starcoin_types::{
    account_address::AccountAddress,
    transaction::{RawUserTransaction, SignedUserTransaction},
};
use std::convert::TryFrom;

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

    fn save_account(&self, account: WalletAccount, key_pair: KeyPair) -> Result<()> {
        let address = account.address;
        self.store.save_account(account)?;
        self.store.save_to_account(
            &address,
            KEY_NAME_PRIVATE_KEY.to_string(),
            key_pair.private_key.to_bytes().to_vec(),
        )?;
        Ok(())
    }

    fn get_key_pair(&self, address: &AccountAddress) -> Result<KeyPair> {
        self.store
            .get_from_account(address, KEY_NAME_PRIVATE_KEY)
            .and_then(|value| {
                value.ok_or(format_err!(
                    "Can not find private_key by address: {:?}",
                    address
                ))
            })
            .and_then(|value| Ok(Ed25519PrivateKey::try_from(value.as_slice())?))
            .and_then(|private_key| Ok(KeyPair::from(private_key)))
    }
}

const KEY_NAME_PRIVATE_KEY: &str = "private_key";

impl<S> Wallet for KeyPairWallet<S>
where
    S: WalletStore,
{
    fn create_account(&self, _password: &str) -> Result<WalletAccount> {
        let mut seed_rng = rand::rngs::OsRng::new().expect("can't access OsRng");
        let seed_buf: [u8; 32] = seed_rng.gen();
        let mut rng: StdRng = SeedableRng::from_seed(seed_buf);
        let key_pair: KeyPair = KeyPair::generate_for_testing(&mut rng);
        let address = AccountAddress::from_public_key(&key_pair.public_key);
        //first account is default.
        let is_default = self.get_accounts()?.len() == 0;
        let account = WalletAccount::new(address, is_default);
        self.save_account(account.clone(), key_pair)?;
        Ok(account)
    }

    fn get_account_detail(&self, address: &AccountAddress) -> Result<Option<AccountDetail>> {
        self.store
            .get_account(address)
            .and_then(|account| match account {
                Some(account) => {
                    let keypair = self.get_key_pair(address)?;
                    Ok(Some(AccountDetail::new(
                        account,
                        keypair.public_key.clone(),
                    )))
                }
                None => Ok(None),
            })
    }

    fn get_account(&self, address: &AccountAddress) -> Result<Option<WalletAccount>> {
        self.store.get_account(address)
    }

    fn import_account(&self, private_key: Vec<u8>, _password: &str) -> Result<WalletAccount> {
        let private_key = Ed25519PrivateKey::try_from(private_key.as_slice())?;
        let key_pair = KeyPair::from(private_key);
        let address = AccountAddress::from_public_key(&key_pair.public_key);
        let account = WalletAccount::new(address, false);
        self.save_account(account.clone(), key_pair)?;
        Ok(account)
    }

    fn contains(&self, address: &AccountAddress) -> Result<bool> {
        Ok(self.store.get_account(address)?.map(|_| true).is_some())
    }

    fn unlock_account(
        &self,
        _address: AccountAddress,
        _password: &str,
        _duration: Duration,
    ) -> Result<()> {
        //do nothing
        Ok(())
    }

    fn lock_account(&self, _address: AccountAddress) -> Result<()> {
        //do nothing
        Ok(())
    }

    fn sign_txn(&self, raw_txn: RawUserTransaction) -> Result<SignedUserTransaction> {
        let address = raw_txn.sender();
        ensure!(
            self.contains(&address)?,
            "Can not find account by address: {:?}",
            address
        );
        let key_pair = self.get_key_pair(&address)?;
        key_pair.sign_txn(raw_txn)
    }

    fn get_default_account(&self) -> Result<Option<WalletAccount>> {
        Ok(self
            .store
            .get_accounts()?
            .iter()
            .find(|account| account.is_default)
            .cloned())
    }

    fn get_accounts(&self) -> Result<Vec<WalletAccount>> {
        self.store.get_accounts()
    }

    fn set_default(&self, address: &AccountAddress) -> Result<()> {
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

        Ok(())
    }

    fn remove_account(&self, address: &AccountAddress) -> Result<()> {
        let account = self.get_account(address)?;
        match account {
            Some(account) => {
                ensure!(!account.is_default, "Can not remove default account.");
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
