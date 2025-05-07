// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::fmt::{Display, Formatter, Pointer};
use crate::account_address::AccountAddress;
use anyhow::{format_err, Error};
use bcs_ext::Sample;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;
use starcoin_vm2_vm_types::{
    account_address::AccountAddress as AccountAddressV2,
    genesis_config::ChainId as ChainIdV2,
    transaction::{
        RawUserTransaction as RawUserTransactionV2,
        SignatureCheckedTransaction as SignatureCheckedTransactionV2,
        SignedUserTransaction as SignedUserTransactionV2,
        TransactionPayload as TransactionPayloadV2,
    },
};
use starcoin_vm_types::transaction::{
    RawUserTransaction, SignatureCheckedTransaction, Transaction, TransactionPayload,
};
use starcoin_vm_types::{genesis_config::ChainId, transaction::SignedUserTransaction};
use crate::multi_transaction_authenticator::MultiTransactionAuthenticator;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub enum MultiChainId {
    VM1(ChainId),
    VM2(ChainIdV2),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub enum MultiSignedUserTransaction {
    VM1(SignedUserTransaction),
    VM2(SignedUserTransactionV2),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub enum MultiTransactionPayload {
    VM1(TransactionPayload),
    VM2(TransactionPayloadV2),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum MultiSignatureCheckedTransaction {
    VM1(SignatureCheckedTransaction),
    VM2(SignatureCheckedTransactionV2),
}

impl MultiSignatureCheckedTransaction {
    pub fn into_inner(self) -> MultiSignedUserTransaction {
        match self {
            MultiSignatureCheckedTransaction::VM1(txn) => {
                MultiSignedUserTransaction::VM1(txn.into_inner())
            }
            MultiSignatureCheckedTransaction::VM2(txn) => {
                MultiSignedUserTransaction::VM2(txn.into_inner())
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum MultiAccountAddress {
    VM1(AccountAddress),
    VM2(AccountAddressV2),
}

impl Into<AccountAddress> for MultiAccountAddress {
    fn into(self) -> AccountAddress {
        match self {
            MultiAccountAddress::VM1(addr) => addr,
            MultiAccountAddress::VM2(addr) => AccountAddress::new(addr.into_bytes()),
        }
    }
}

impl Display for MultiAccountAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MultiAccountAddress::VM1(addr) => addr.fmt(f),
            MultiAccountAddress::VM2(addr) => addr.fmt(f)
        }
    }
}

// impl From<AccountAddressV2> for MultiAccountAddress {
//     fn from(addr: MultiAccountAddress) -> Self {
//         match addr {
//             MultiAccountAddress::VM1(addr) => AccountAddressV2::new(addr.into_bytes()),
//             MultiAccountAddress::VM2(addr) => addr,
//         }
//     }
// }

impl Sample for MultiSignedUserTransaction {
    fn sample() -> Self {
        Self::VM1(SignedUserTransaction::sample())
    }
}

impl From<SignedUserTransaction> for MultiSignedUserTransaction {
    fn from(txn: SignedUserTransaction) -> Self {
        Self::VM1(txn)
    }
}

impl From<SignedUserTransactionV2> for MultiSignedUserTransaction {
    fn from(txn: SignedUserTransactionV2) -> Self {
        Self::VM2(txn)
    }
}

impl MultiSignedUserTransaction {
    pub fn id(&self) -> HashValue {
        match self {
            Self::VM1(sign) => sign.id(),
            MultiSignedUserTransaction::VM2(sign) => HashValue::new(sign.id().to_inner()),
        }
    }

    pub fn sequence_number(&self) -> u64 {
        match self {
            Self::VM1(sign) => sign.sequence_number(),
            Self::VM2(sign) => sign.sequence_number(),
        }
    }

    pub fn expiration_timestamp_secs(&self) -> u64 {
        match self {
            Self::VM1(sign) => sign.expiration_timestamp_secs(),
            Self::VM2(sign) => sign.expiration_timestamp_secs(),
        }
    }

    pub fn check_signature(self) -> Result<MultiSignatureCheckedTransaction, anyhow::Error> {
        match self {
            Self::VM1(sign) => sign
                .check_signature()
                .map(MultiSignatureCheckedTransaction::VM1),
            Self::VM2(sign) => sign
                .check_signature()
                .map(MultiSignatureCheckedTransaction::VM2),
        }
    }


    pub fn authenticator(&self) -> MultiTransactionAuthenticator {
        match self {
            Self::VM1(sign) => MultiTransactionAuthenticator::VM1(sign.authenticator()),
            Self::VM2(sign_with_type) => MultiTransactionAuthenticator::VM2(sign_with_type.authenticator()),
        }
    }

    pub fn sender(&self) -> MultiAccountAddress {
        match self {
            Self::VM1(sign) => MultiAccountAddress::VM1(sign.sender()),
            Self::VM2(sign) => MultiAccountAddress::VM2(sign.sender()),
        }
    }

    pub fn max_gas_amount(&self) -> u64 {
        match self {
            Self::VM1(sign) => sign.max_gas_amount(),
            Self::VM2(sign) => sign.max_gas_amount(),
        }
    }

    pub fn gas_unit_price(&self) -> u64 {
        match self {
            Self::VM1(sign) => sign.gas_unit_price(),
            Self::VM2(sign) => sign.gas_unit_price(),
        }
    }

    pub fn gas_token_code(&self) -> String {
        match self {
            Self::VM1(sign) => sign.gas_token_code().to_string(),
            Self::VM2(sign) => sign.gas_token_code().to_string(),
        }
    }

    pub fn chain_id(&self) -> MultiChainId {
        match self {
            Self::VM1(sign) => MultiChainId::VM1(sign.chain_id()),
            Self::VM2(sign) => MultiChainId::VM2(sign.chain_id()),
        }
    }

    pub fn payload(&self) -> MultiTransactionPayload {
        match self {
            Self::VM1(sign) => MultiTransactionPayload::VM1(sign.payload().clone()),
            Self::VM2(sign) => MultiTransactionPayload::VM2(sign.payload().clone()),
        }
    }

    pub fn raw_txn_bytes_len(&self) -> usize {
        match self {
            Self::VM1(sign) => sign.raw_txn_bytes_len(),
            Self::VM2(sign) => sign.raw_txn_bytes_len(),
        }
    }

    pub fn mock() -> Self {
        Self::VM1(SignedUserTransaction::mock())
    }
}

impl TryFrom<Transaction> for MultiSignedUserTransaction {
    type Error = Error;

    fn try_from(txn: Transaction) -> Result<Self, Self::Error> {
        match txn {
            Transaction::UserTransaction(txn) => Ok(Self::VM1(txn)),
            _ => Err(format_err!("Not a user transaction.")),
        }
    }
}

impl From<MultiSignedUserTransaction> for Transaction {
    fn from(txn: MultiSignedUserTransaction) -> Self {
        match txn {
            MultiSignedUserTransaction::VM1(txn) => Transaction::UserTransaction(txn),
            _ => panic!("Not a vm1 transaction."),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum MultiRawUserTransaction {
    VM1(RawUserTransaction),
    VM2(RawUserTransactionV2),
}

impl MultiRawUserTransaction {
    pub fn into_payload(self) -> MultiTransactionPayload {
        match self {
            MultiRawUserTransaction::VM1(raw_txn) => {
                MultiTransactionPayload::VM1(raw_txn.payload().clone())
            }
            MultiRawUserTransaction::VM2(raw_txn) => {
                MultiTransactionPayload::VM2(raw_txn.payload().clone())
            }
        }
    }

    /// Return the sender of this transaction.
    pub fn sender(&self) -> MultiAccountAddress {
        match self {
            MultiRawUserTransaction::VM1(raw_txn) => {
                MultiAccountAddress::VM1(raw_txn.sender().clone())
            }
            MultiRawUserTransaction::VM2(raw_txn) => {
                MultiAccountAddress::VM2(raw_txn.sender().clone())
            }
        }
    }
    pub fn sequence_number(&self) -> u64 {
        match self {
            MultiRawUserTransaction::VM1(raw_txn) => raw_txn.sequence_number(),
            MultiRawUserTransaction::VM2(raw_txn) => raw_txn.sequence_number(),
        }
    }
    pub fn max_gas_amount(&self) -> u64 {
        match self {
            MultiRawUserTransaction::VM1(raw_txn) => raw_txn.max_gas_amount(),
            MultiRawUserTransaction::VM2(raw_txn) => raw_txn.max_gas_amount(),
        }
    }
    pub fn gas_unit_price(&self) -> u64 {
        match self {
            MultiRawUserTransaction::VM1(raw_txn) => raw_txn.gas_unit_price(),
            MultiRawUserTransaction::VM2(raw_txn) => raw_txn.gas_unit_price(),
        }
    }
    pub fn gas_token_code(&self) -> String {
        match self {
            MultiRawUserTransaction::VM1(raw_txn) => raw_txn.gas_token_code(),
            MultiRawUserTransaction::VM2(raw_txn) => raw_txn.gas_token_code(),
        }
    }
    pub fn expiration_timestamp_secs(&self) -> u64 {
        match self {
            MultiRawUserTransaction::VM1(raw_txn) => raw_txn.expiration_timestamp_secs(),
            MultiRawUserTransaction::VM2(raw_txn) => raw_txn.expiration_timestamp_secs(),
        }
    }
    pub fn chain_id(&self) -> MultiChainId {
        match self {
            MultiRawUserTransaction::VM1(raw_txn) => MultiChainId::VM1(raw_txn.chain_id()),
            MultiRawUserTransaction::VM2(raw_txn) => MultiChainId::VM2(raw_txn.chain_id()),
        }
    }
    pub fn payload(&self) -> MultiTransactionPayload {
        match self {
            MultiRawUserTransaction::VM1(raw_txn) => {
                MultiTransactionPayload::VM1(raw_txn.payload().clone())
            }
            MultiRawUserTransaction::VM2(raw_txn) => {
                MultiTransactionPayload::VM2(raw_txn.payload().clone())
            }
        }
    }

    pub fn txn_size(&self) -> usize {
        match self {
            MultiRawUserTransaction::VM1(raw_txn) => bcs_ext::to_bytes(raw_txn)
                .expect("Unable to serialize RawUserTransaction")
                .len(),
            MultiRawUserTransaction::VM2(raw_txn) => bcs_ext::to_bytes(raw_txn)
                .expect("Unable to serialize RawUserTransaction")
                .len(),
        }
    }

    // pub fn from_hex<T: AsRef<[u8]>>(hex: T) -> anyhow::Result<Self> {
    //     Self::from_bytes(hex::decode(hex)?)
    // }
    //
    // pub fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> anyhow::Result<Self> {
    //     bcs_ext::from_bytes(bytes.as_ref())
    // }

    pub fn to_hex(&self) -> String {
        match self {
            MultiRawUserTransaction::VM1(raw_txn) => format!(
                "0x{}",
                hex::encode(
                    bcs_ext::to_bytes(&raw_txn)
                        .expect("Serialize RawUserTransaction should success.")
                )
            ),
            MultiRawUserTransaction::VM2(raw_txn) => format!(
                "0x{}",
                hex::encode(
                    bcs_ext::to_bytes(&raw_txn)
                        .expect("Serialize RawUserTransaction should success.")
                )
            ),
        }
    }
}
