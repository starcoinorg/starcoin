// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::error::AccountServiceError;
use crate::{AccountAsyncService, AccountInfo, ServiceResult};
use anyhow::Result;
use dashmap::DashMap;
use starcoin_crypto::ed25519::{Ed25519PrivateKey, Ed25519PublicKey};
use starcoin_crypto::keygen::KeyGen;
use starcoin_crypto::PrivateKey;
use starcoin_types::account_address::{self, AccountAddress};
use starcoin_types::account_config::token_code::TokenCode;
use starcoin_types::transaction::{RawUserTransaction, SignedUserTransaction};
use std::convert::TryFrom;
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone)]
pub struct MockAccountService {
    accounts: Arc<DashMap<AccountAddress, MockAccountData>>,
}

#[derive(Debug, Clone)]
struct MockAccountData {
    private_key: Ed25519PrivateKey,
    public_key: Ed25519PublicKey,
    is_default: bool,
    pass: String,
}

impl MockAccountService {
    pub fn new() -> Result<Self> {
        Ok(Self {
            accounts: Arc::new(DashMap::default()),
        })
    }

    pub fn get_account_info(&self, addr: AccountAddress) -> Option<AccountInfo> {
        let kv = self.accounts.get(&addr)?;
        Some(AccountInfo::new(addr, kv.public_key.clone(), kv.is_default))
    }
}

#[async_trait::async_trait]
impl AccountAsyncService for MockAccountService {
    async fn create_account(&self, password: String) -> ServiceResult<AccountInfo> {
        let mut key_gen = KeyGen::from_os_rng();
        let (private_key, public_key) = key_gen.generate_keypair();
        let addr = account_address::from_public_key(&public_key);
        let is_default = self.accounts.is_empty();
        self.accounts.insert(
            addr,
            MockAccountData {
                pass: password,
                private_key,
                public_key,
                is_default,
            },
        );
        Ok(self.get_account_info(addr).unwrap())
    }

    async fn get_default_account(&self) -> ServiceResult<Option<AccountInfo>> {
        for r in self.accounts.as_ref() {
            if r.is_default {
                return Ok(Some(AccountInfo {
                    address: *r.key(),
                    is_default: r.is_default,
                    public_key: r.public_key.clone(),
                }));
            }
        }
        Ok(None)
    }

    async fn set_default_account(
        &self,
        address: AccountAddress,
    ) -> ServiceResult<Option<AccountInfo>> {
        for mut r in self.accounts.iter_mut() {
            if r.is_default {
                r.is_default = false;
            }
        }
        match self.accounts.get_mut(&address) {
            None => Ok(None),
            Some(mut account) => {
                account.is_default = true;
                Ok(Some(AccountInfo {
                    address: *account.key(),
                    is_default: true,
                    public_key: account.public_key.clone(),
                }))
            }
        }
    }

    async fn get_accounts(&self) -> ServiceResult<Vec<AccountInfo>> {
        Ok(self
            .accounts
            .iter()
            .map(|r| AccountInfo {
                address: *r.key(),
                is_default: r.is_default,
                public_key: r.public_key.clone(),
            })
            .collect())
    }

    async fn get_account(&self, address: AccountAddress) -> ServiceResult<Option<AccountInfo>> {
        Ok(self.get_account_info(address))
    }

    async fn sign_txn(
        &self,
        raw_txn: RawUserTransaction,
        signer_address: AccountAddress,
    ) -> ServiceResult<SignedUserTransaction> {
        let r = self
            .accounts
            .get(&signer_address)
            .ok_or_else(|| AccountServiceError::AccountNotExist(signer_address))?;
        Ok(raw_txn
            .sign(&r.private_key, r.public_key.clone())
            .unwrap()
            .into_inner())
    }

    async fn unlock_account(
        &self,
        _address: AccountAddress,
        _password: String,
        _duration: Duration,
    ) -> ServiceResult<()> {
        Ok(())
    }
    async fn lock_account(&self, _address: AccountAddress) -> ServiceResult<()> {
        Ok(())
    }

    async fn import_account(
        &self,
        address: AccountAddress,
        private_key: Vec<u8>,
        password: String,
    ) -> ServiceResult<AccountInfo> {
        let private_key = Ed25519PrivateKey::try_from(private_key.as_slice())
            .map_err(|e| AccountServiceError::OtherError(e.into()))?;
        let public_key = private_key.public_key();
        self.accounts.insert(
            address,
            MockAccountData {
                private_key,
                public_key,
                is_default: false,
                pass: password,
            },
        );
        Ok(self.get_account_info(address).unwrap())
    }

    /// Return the private key as bytes for `address`
    async fn export_account(
        &self,
        address: AccountAddress,
        _password: String,
    ) -> ServiceResult<Vec<u8>> {
        let r = self
            .accounts
            .get(&address)
            .ok_or_else(|| AccountServiceError::AccountNotExist(address))?;
        Ok(r.private_key.to_bytes().to_vec())
    }

    async fn accepted_tokens(&self, _address: AccountAddress) -> ServiceResult<Vec<TokenCode>> {
        Ok(vec![])
    }
}
