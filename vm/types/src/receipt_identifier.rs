// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_address::AccountAddress;
use crate::transaction::authenticator::AuthenticationKey;
use anyhow::Result;
use bech32::ToBase32;
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::convert::TryFrom;
use std::fmt::Formatter;
use std::str::FromStr;

/// See sip-21
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum ReceiptIdentifier {
    V1(AccountAddress, Option<AuthenticationKey>),
}

impl FromStr for ReceiptIdentifier {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::decode(s)
    }
}
impl std::fmt::Display for ReceiptIdentifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.encode())
    }
}

impl ReceiptIdentifier {
    pub fn v1(address: AccountAddress, auth_key: Option<AuthenticationKey>) -> ReceiptIdentifier {
        ReceiptIdentifier::V1(address, auth_key)
    }
    pub fn address(&self) -> AccountAddress {
        match self {
            ReceiptIdentifier::V1(address, _) => *address,
        }
    }
    pub fn auth_key(&self) -> Option<&AuthenticationKey> {
        match self {
            ReceiptIdentifier::V1(_, auth_key) => auth_key.as_ref(),
        }
    }
    pub fn encode(&self) -> String {
        match self {
            ReceiptIdentifier::V1(address, auth_key) => {
                let mut data = vec![];
                data.append(address.to_vec().as_mut());
                if let Some(auth_key) = auth_key {
                    data.append(auth_key.to_vec().as_mut());
                }

                let mut data = data.to_base32();
                data.insert(0, bech32::u5::try_from_u8(1).unwrap());
                bech32::encode("stc", data, bech32::Variant::Bech32).unwrap()
            }
        }
    }
    pub fn decode(s: impl AsRef<str>) -> Result<ReceiptIdentifier> {
        #![allow(clippy::integer_arithmetic)]

        let (hrp, data, variant) = bech32::decode(s.as_ref()).unwrap();

        anyhow::ensure!(variant == bech32::Variant::Bech32, "expect bech32 encoding");
        anyhow::ensure!(hrp.as_str() == "stc", "expect bech32 hrp to be stc");

        let version = data.first().map(|u| u.to_u8());
        anyhow::ensure!(version.filter(|v| *v == 1u8).is_some(), "expect version 1");

        let data: Vec<u8> = bech32::FromBase32::from_base32(&data[1..])?;

        let (address, auth_key) = if data.len() == AccountAddress::LENGTH {
            (AccountAddress::from_bytes(data.as_slice())?, None)
        } else if data.len() == AccountAddress::LENGTH + AuthenticationKey::LENGTH {
            let address = AccountAddress::from_bytes(&data[0..AccountAddress::LENGTH])?;
            let auth_key = AuthenticationKey::try_from(&data[AccountAddress::LENGTH..])?;
            (address, Some(auth_key))
        } else {
            anyhow::bail!("invalid data");
        };
        Ok(ReceiptIdentifier::V1(address, auth_key))
    }
}

impl<'de> Deserialize<'de> for ReceiptIdentifier {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = <String>::deserialize(deserializer)?;
        ReceiptIdentifier::decode(s).map_err(D::Error::custom)
    }
}

impl Serialize for ReceiptIdentifier {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.encode().as_str())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    pub fn test_rust_bench32() {
        let address = AccountAddress::random();
        let auth_key = AuthenticationKey::random();

        let encoded = ReceiptIdentifier::V1(address, Some(auth_key)).to_string();
        println!(
            "address: {}, auth_key: {}, id: {}",
            address, auth_key, &encoded
        );

        let id = ReceiptIdentifier::from_str(encoded.as_str()).unwrap();
        match id {
            ReceiptIdentifier::V1(decoded_address, decoded_auth_key) => {
                assert_eq!(decoded_address, address);
                assert_eq!(decoded_auth_key, Some(auth_key));
            }
        }
    }
}
