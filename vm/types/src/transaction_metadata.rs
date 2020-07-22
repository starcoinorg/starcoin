// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

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
use std::{convert::TryFrom, time::Duration};

pub struct TransactionMetadata {
    pub sender: AccountAddress,
    pub authentication_key_preimage: Vec<u8>,
    pub sequence_number: u64,
    pub max_gas_amount: GasUnits<GasCarrier>,
    pub gas_unit_price: GasPrice<GasCarrier>,
    pub transaction_size: AbstractMemorySize<GasCarrier>,
    pub expiration_time: Duration,
    //TODO refactor this when can pass struct to Move script,
    // use PayloadMetadata to wrap there's fields.
    pub payload_type: TransactionPayloadType,
    pub script_or_package_hash: HashValue,
    pub package_address: Option<AccountAddress>,
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
            transaction_size: AbstractMemorySize::new(txn.raw_txn_bytes_len() as u64),
            expiration_time: txn.expiration_time(),
            payload_type: txn.payload().payload_type(),
            script_or_package_hash: match txn.payload() {
                TransactionPayload::Script(script) => HashValue::sha3_256_of(script.code()),
                TransactionPayload::Package(package) => package.crypto_hash(),
            },
            package_address: match txn.payload() {
                TransactionPayload::Script(_script) => None,
                TransactionPayload::Package(package) => Some(package.package_address()),
            },
        }
    }

    pub fn max_gas_amount(&self) -> GasUnits<GasCarrier> {
        self.max_gas_amount
    }

    pub fn gas_unit_price(&self) -> GasPrice<GasCarrier> {
        self.gas_unit_price
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

    pub fn expiration_time(&self) -> u64 {
        self.expiration_time.as_secs()
    }

    pub fn payload_type(&self) -> TransactionPayloadType {
        self.payload_type
    }

    pub fn script_or_package_hash(&self) -> HashValue {
        self.script_or_package_hash
    }
}

impl Default for TransactionMetadata {
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
            transaction_size: AbstractMemorySize::new(0),
            expiration_time: Duration::new(0, 0),
            payload_type: TransactionPayloadType::Script,
            script_or_package_hash: HashValue::zero(),
            package_address: None,
        }
    }
}
