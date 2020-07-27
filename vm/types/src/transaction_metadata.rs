// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_config::STC_TOKEN_CODE;
use crate::token::token_code::TokenCode;
use crate::{
    account_address::AccountAddress,
    transaction::{authenticator::AuthenticationKeyPreimage, SignedUserTransaction},
    transaction::{TransactionPayload, TransactionPayloadType},
};
use move_core_types::gas_schedule::{
    AbstractMemorySize, GasAlgebra, GasCarrier, GasPrice, GasUnits,
};
use starcoin_crypto::hash::PlainCryptoHash;
use starcoin_crypto::{ed25519::Ed25519PrivateKey, HashValue, PrivateKey};
use std::convert::TryFrom;
use std::str::FromStr;

pub enum TransactionPayloadMetadata {
    Script(HashValue),
    Package(HashValue, AccountAddress),
}

impl TransactionPayloadMetadata {
    pub fn payload_type(&self) -> TransactionPayloadType {
        match self {
            TransactionPayloadMetadata::Script(_) => TransactionPayloadType::Script,
            TransactionPayloadMetadata::Package(_, _) => TransactionPayloadType::Package,
        }
    }
}

pub struct TransactionMetadata {
    pub sender: AccountAddress,
    pub authentication_key_preimage: Vec<u8>,
    pub sequence_number: u64,
    pub max_gas_amount: GasUnits<GasCarrier>,
    pub gas_unit_price: GasPrice<GasCarrier>,
    pub gas_token_code: TokenCode,
    pub transaction_size: AbstractMemorySize<GasCarrier>,
    pub expiration_timestamp_secs: u64,
    pub payload: TransactionPayloadMetadata,
}

impl TransactionMetadata {
    pub fn new(txn: &SignedUserTransaction) -> Self {
        Self {
            sender: txn.sender(),
            authentication_key_preimage: txn
                .authenticator()
                .authentication_key_preimage()
                .into_vec(),
            sequence_number: txn.sequence_number(),
            max_gas_amount: GasUnits::new(txn.max_gas_amount()),
            gas_unit_price: GasPrice::new(txn.gas_unit_price()),
            gas_token_code: TokenCode::from_str(txn.gas_token_code())
                .expect("Transaction's gas_token_code must been verified at TransactionBuilder."),
            transaction_size: AbstractMemorySize::new(txn.raw_txn_bytes_len() as u64),
            expiration_timestamp_secs: txn.expiration_timestamp_secs(),
            payload: match txn.payload() {
                TransactionPayload::Script(script) => {
                    TransactionPayloadMetadata::Script(HashValue::sha3_256_of(script.code()))
                }
                TransactionPayload::Package(package) => TransactionPayloadMetadata::Package(
                    package.crypto_hash(),
                    package.package_address(),
                ),
            },
        }
    }

    pub fn max_gas_amount(&self) -> GasUnits<GasCarrier> {
        self.max_gas_amount
    }

    pub fn gas_unit_price(&self) -> GasPrice<GasCarrier> {
        self.gas_unit_price
    }

    pub fn gas_token_code(&self) -> TokenCode {
        self.gas_token_code.clone()
    }

    pub fn sender(&self) -> AccountAddress {
        self.sender.to_owned()
    }

    pub fn authentication_key_preimage(&self) -> &[u8] {
        &self.authentication_key_preimage
    }

    pub fn sequence_number(&self) -> u64 {
        self.sequence_number
    }

    pub fn transaction_size(&self) -> AbstractMemorySize<GasCarrier> {
        self.transaction_size
    }

    pub fn expiration_time_secs(&self) -> u64 {
        self.expiration_timestamp_secs
    }

    pub fn payload_type(&self) -> TransactionPayloadType {
        self.payload.payload_type()
    }

    pub fn payload(&self) -> &TransactionPayloadMetadata {
        &self.payload
    }
}

impl Default for TransactionMetadata {
    //TODO remove this default construct.
    fn default() -> Self {
        let mut buf = [0u8; Ed25519PrivateKey::LENGTH];
        buf[Ed25519PrivateKey::LENGTH - 1] = 1;
        let public_key = Ed25519PrivateKey::try_from(&buf[..]).unwrap().public_key();
        TransactionMetadata {
            sender: AccountAddress::ZERO,
            authentication_key_preimage: AuthenticationKeyPreimage::ed25519(&public_key).into_vec(),
            sequence_number: 0,
            max_gas_amount: GasUnits::new(100_000_000),
            gas_unit_price: GasPrice::new(0),
            gas_token_code: STC_TOKEN_CODE.clone(),
            transaction_size: AbstractMemorySize::new(0),
            expiration_timestamp_secs: 0,
            payload: TransactionPayloadMetadata::Script(HashValue::zero()),
        }
    }
}
