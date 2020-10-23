// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_manager::gen_private_key;
use crate::account_storage::AccountStorage;
use anyhow::{format_err, Result};
use starcoin_account_api::error::AccountError;
use starcoin_account_api::{AccountInfo, AccountPrivateKey, AccountPublicKey, AccountResult};
use starcoin_crypto::PrivateKey;
use starcoin_storage::storage::StorageInstance;
use starcoin_types::account_address;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::transaction::authenticator::AuthenticationKey;
use starcoin_types::transaction::{RawUserTransaction, SignedUserTransaction};

pub struct Account {
    addr: AccountAddress,
    private_key: AccountPrivateKey,
    store: AccountStorage,
}

impl Account {
    pub fn create(
        private_key: AccountPrivateKey,
        addr: Option<AccountAddress>,
        password: String,
        storage: AccountStorage,
    ) -> AccountResult<Self> {
        let address = addr.unwrap_or_else(|| private_key.public_key().derived_address());

        storage.update_key(address, &private_key, password.as_str())?;

        Ok(Self {
            addr: address,
            private_key,
            store: storage,
        })
    }

    pub fn load(
        addr: AccountAddress,
        password: &str,
        store: AccountStorage,
    ) -> AccountResult<Option<Self>> {
        let decrypted_key = store.decrypt_private_key(addr, password)?;
        let private_key = match decrypted_key {
            None => return Ok(None),
            Some(p) => p,
        };

        let saved_public_key = store.public_key(addr)?;
        let saved_public_key = saved_public_key.ok_or_else(|| {
            AccountError::StoreError(format_err!("public key not found for address {}", addr))
        })?;
        if saved_public_key.to_bytes() != private_key.public_key().to_bytes() {
            return Err(AccountError::StoreError(format_err!(
                "invalid state of public key and private key"
            )));
        }

        Ok(Some(Self {
            addr,
            private_key,
            store,
        }))
    }

    pub fn info(&self) -> AccountInfo {
        // TODO: fix is_default
        AccountInfo::new(self.addr, self.private_key.public_key(), false)
    }

    pub fn sign_txn(&self, raw_txn: RawUserTransaction) -> Result<SignedUserTransaction> {
        //TODO handle multi signature
        let signature = self.private_key.sign(&raw_txn);
        signature.build_transaction(raw_txn)
    }

    pub fn destroy(self) -> Result<()> {
        self.store.destroy_account(self.addr)
    }

    pub fn address(&self) -> &AccountAddress {
        &self.addr
    }

    pub fn private_key(&self) -> &AccountPrivateKey {
        &self.private_key
    }

    pub fn public_key(&self) -> AccountPublicKey {
        self.private_key.public_key()
    }

    pub fn auth_key(&self) -> AuthenticationKey {
        self.public_key().auth_key()
    }

    ///Generate a random account for test.
    pub fn random() -> Result<Self> {
        let private_key = gen_private_key();
        let public_key = private_key.public_key();
        let address = account_address::from_public_key(&public_key);
        let storage = AccountStorage::new(StorageInstance::new_cache_instance());
        Self::create(private_key.into(), Some(address), "".to_string(), storage)
            .map_err(|e| e.into())
    }
}
