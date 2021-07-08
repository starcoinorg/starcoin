// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_address::AccountAddress;
use crate::account_config::AccountResource;
use crate::transaction::authenticator::{AuthenticationKey, TransactionAuthenticator};
use anyhow::{ensure, Error, Result};
use serde::{Deserialize, Serialize};
use starcoin_crypto::hash::{CryptoHash, CryptoHasher};
use std::fmt::Formatter;
use std::str::FromStr;

/// SigningMessage is a message to be signed and encapsulates the salt
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, CryptoHasher, CryptoHash)]
pub struct SigningMessage {
    message: Vec<u8>,
}

impl FromStr for SigningMessage {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        ensure!(!s.is_empty(), "signing message should not be empty.",);
        Ok(Self {
            message: s.as_bytes().to_vec(),
        })
    }
}

/// A message has signed
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, CryptoHasher, CryptoHash)]
pub struct SignedMessage {
    /// The account to sign the message.
    pub account: AccountAddress,
    pub message: SigningMessage,
    pub authenticator: TransactionAuthenticator,
}

impl SignedMessage {
    pub fn new(
        account: AccountAddress,
        message: SigningMessage,
        authenticator: TransactionAuthenticator,
    ) -> Self {
        Self {
            account,
            message,
            authenticator,
        }
    }
    /// Checks that the signature of given message. Returns `Ok()` if the signature is valid.
    /// Note: this method do not check the relation of account and public key.
    pub fn check_signature(&self) -> Result<()> {
        self.authenticator.verify(&self.message)
    }

    /// Checks the account by on chain account resource, please ensure the AccountResource's address == message.account
    /// if the `account_resource` is None, it means that the account is not create on chain
    pub fn check_account(&self, account_resource: Option<&AccountResource>) -> Result<()> {
        let authkey_in_message = self.authenticator.authentication_key();
        let authkey_on_chain = account_resource
            .map(|account| account.authentication_key())
            .unwrap_or_else(|| AuthenticationKey::DUMMY_KEY.as_ref());
        // if the account not exist on chain or the authentication_key on chain is dummy key, just check the derived_address.
        if authkey_on_chain == AuthenticationKey::DUMMY_KEY.as_ref() {
            ensure!(
                authkey_in_message.derived_address() == self.account,
                "authenticator's address do not match the signed message account"
            )
        } else {
            ensure!(
                authkey_in_message.as_ref() == authkey_on_chain,
                "authenticator's public key do not match the account resource on chain"
            );
        }
        Ok(())
    }

    pub fn to_hex(&self) -> String {
        hex::encode(bcs_ext::to_bytes(self).expect("SignedMessage bcs serialize should success."))
    }
}

impl FromStr for SignedMessage {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.strip_prefix("0x").unwrap_or(s);
        let bytes = hex::decode(s)?;
        bcs_ext::from_bytes(bytes.as_slice())
    }
}

impl std::fmt::Display for SignedMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{}", self.to_hex())
    }
}
