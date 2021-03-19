// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::genesis_config::ChainId;
use crate::token::token_code::TokenCode;
use crate::transaction::authenticator::AuthenticationKeyPreimage;
use crate::transaction::RawUserTransaction;
use crate::vm_status::{StatusCode, VMStatus};
use crate::{
    account_address::AccountAddress,
    transaction::SignedUserTransaction,
    transaction::{TransactionPayload, TransactionPayloadType},
};
use anyhow::Result;
use move_core_types::gas_schedule::{
    AbstractMemorySize, GasAlgebra, GasCarrier, GasPrice, GasUnits,
};
use starcoin_crypto::hash::PlainCryptoHash;
use starcoin_crypto::HashValue;
use std::str::FromStr;

pub enum TransactionPayloadMetadata {
    Script(HashValue),
    Package(HashValue, AccountAddress),
    ScriptFunction,
}

impl TransactionPayloadMetadata {
    pub fn payload_type(&self) -> TransactionPayloadType {
        match self {
            TransactionPayloadMetadata::Script(_) => TransactionPayloadType::Script,
            TransactionPayloadMetadata::Package(_, _) => TransactionPayloadType::Package,
            TransactionPayloadMetadata::ScriptFunction => TransactionPayloadType::ScriptFunction,
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
    pub chain_id: ChainId,
    pub payload: TransactionPayloadMetadata,
}

impl TransactionMetadata {
    pub fn new(txn: &SignedUserTransaction) -> Result<Self, VMStatus> {
        Self::from_raw_txn_and_preimage(
            txn.raw_txn(),
            txn.authenticator().authentication_key_preimage(),
        )
    }

    pub fn from_raw_txn_and_preimage(
        txn: &RawUserTransaction,
        auth_preimage: AuthenticationKeyPreimage,
    ) -> Result<Self, VMStatus> {
        Ok(Self {
            sender: txn.sender(),
            authentication_key_preimage: auth_preimage.into_vec(),
            sequence_number: txn.sequence_number(),
            max_gas_amount: GasUnits::new(txn.max_gas_amount()),
            gas_unit_price: GasPrice::new(txn.gas_unit_price()),
            gas_token_code: TokenCode::from_str(txn.gas_token_code().as_str())
                .map_err(|_e| VMStatus::Error(StatusCode::BAD_TRANSACTION_FEE_CURRENCY))?,
            transaction_size: AbstractMemorySize::new(txn.txn_size() as u64),
            expiration_timestamp_secs: txn.expiration_timestamp_secs(),
            chain_id: txn.chain_id(),
            payload: match txn.payload() {
                TransactionPayload::Script(script) => {
                    TransactionPayloadMetadata::Script(HashValue::sha3_256_of(script.code()))
                }
                TransactionPayload::Package(package) => TransactionPayloadMetadata::Package(
                    package.crypto_hash(),
                    package.package_address(),
                ),
                TransactionPayload::ScriptFunction(_) => TransactionPayloadMetadata::ScriptFunction,
            },
        })
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

    pub fn chain_id(&self) -> ChainId {
        self.chain_id
    }

    pub fn payload_type(&self) -> TransactionPayloadType {
        self.payload.payload_type()
    }

    pub fn payload(&self) -> &TransactionPayloadMetadata {
        &self.payload
    }
}
