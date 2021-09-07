// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_address::AccountAddress;
use crate::account_config::AccountResource;
use crate::genesis_config::ChainId;
use crate::transaction::authenticator::{AuthenticationKey, TransactionAuthenticator};
use anyhow::{ensure, Result};
use schemars::{self, JsonSchema};
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use starcoin_crypto::hash::{CryptoHash, CryptoHasher};
use std::fmt::Formatter;
use std::str::FromStr;

/// SigningMessage is a message to be signed and encapsulates the salt
#[derive(Clone, Debug, Hash, Eq, PartialEq, CryptoHasher, CryptoHash, JsonSchema)]
pub struct SigningMessage(#[schemars(with = "String")] pub Vec<u8>);

impl std::fmt::Display for SigningMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{}", hex::encode(self.0.as_slice()))
    }
}

impl FromStr for SigningMessage {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        ensure!(!s.is_empty(), "signing message should not be empty.",);
        Ok(match s.strip_prefix("0x") {
            Some(hex) => Self(hex::decode(hex)?),
            None => Self(s.as_bytes().to_vec()),
        })
    }
}

impl From<Vec<u8>> for SigningMessage {
    fn from(v: Vec<u8>) -> Self {
        Self(v)
    }
}

impl<'de> Deserialize<'de> for SigningMessage {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let s = <String>::deserialize(deserializer)?;
            SigningMessage::from_str(&s).map_err(D::Error::custom)
        } else {
            // In order to preserve the Serde data model and help analysis tools,
            // make sure to wrap our value in a container with the same name
            // as the original type.
            #[derive(::serde::Deserialize)]
            #[serde(rename = "SigningMessage")]
            struct Value(Vec<u8>);

            let value = Value::deserialize(deserializer)?;
            Ok(SigningMessage(value.0))
        }
    }
}

impl Serialize for SigningMessage {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            self.to_string().serialize(serializer)
        } else {
            // See comment in deserialize.
            serializer.serialize_newtype_struct("SigningMessage", &self.0)
        }
    }
}

/// A message has signed
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, CryptoHasher, CryptoHash)]
pub struct SignedMessage {
    /// The account to sign the message.
    pub account: AccountAddress,
    pub message: SigningMessage,
    pub authenticator: TransactionAuthenticator,
    pub chain_id: ChainId,
}

impl SignedMessage {
    pub fn new(
        account: AccountAddress,
        message: SigningMessage,
        authenticator: TransactionAuthenticator,
        chain_id: ChainId,
    ) -> Self {
        Self {
            account,
            message,
            authenticator,
            chain_id,
        }
    }
    /// Checks that the signature of given message. Returns `Ok()` if the signature is valid.
    /// Note: this method do not check the relation of account and public key.
    pub fn check_signature(&self) -> Result<()> {
        self.authenticator.verify(&self.message)
    }

    /// Checks the account by on chain account resource, please ensure the AccountResource's address == message.account
    /// `chain_id` is the chain of the account resource on.
    /// if the `account_resource` is None, it means that the account is not create on chain
    pub fn check_account(
        &self,
        chain_id: ChainId,
        account_resource: Option<&AccountResource>,
    ) -> Result<()> {
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
                self.chain_id == chain_id,
                "The chain id in message and on chain account miss match."
            );
            ensure!(
                authkey_in_message.as_ref() == authkey_on_chain,
                "authenticator's public key do not match the account resource on chain"
            );
        }
        Ok(())
    }

    pub fn to_hex(&self) -> String {
        format!(
            "0x{}",
            hex::encode(
                bcs_ext::to_bytes(self).expect("SignedMessage bcs serialize should success.")
            )
        )
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
        write!(f, "{}", self.to_hex())
    }
}
