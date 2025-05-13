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
        SignatureCheckedTransaction as SignatureCheckedTransactionV2,
        SignedUserTransaction as SignedUserTransactionV2, Transaction as TransactionV2,
        TransactionPayload as TransactionPayloadV2,
    },
};
use starcoin_vm_types::transaction::{
    SignatureCheckedTransaction, Transaction, TransactionPayload,
};
use starcoin_vm_types::{genesis_config::ChainId, transaction::SignedUserTransaction};

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

    /*
    pub fn authenticator(&self) -> TransactionAuthenticator {
        match self {
            Self::VM1(sign) => sign.authenticator(),
            Self::VM2(sign_with_type) => sign_with_type.authenticator(),
        }
    } */

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

impl TryFrom<TransactionV2> for MultiSignedUserTransaction {
    type Error = Error;

    fn try_from(txn: TransactionV2) -> Result<Self, Self::Error> {
        match txn {
            TransactionV2::UserTransaction(txn) => Ok(Self::VM2(txn)),
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
