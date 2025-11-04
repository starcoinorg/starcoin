use crate::transaction::StcTransaction;
use crate::{account_address::AccountAddress, multi_access_path::MultiAccessPath};
use anyhow::{format_err, Error};
use bcs_ext::Sample;
use move_vm2_core_types::move_resource::MoveStructType;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;
use starcoin_vm2_types::access_path::AccessPath as AccessPath2;
use starcoin_vm2_types::account_config::AccountResource as AccountResource2;
use starcoin_vm2_vm_types::{
    account_address::AccountAddress as AccountAddress2,
    transaction::{
        SignatureCheckedTransaction as SignatureCheckedTransaction2,
        SignedUserTransaction as SignedUserTransaction2, Transaction as Transaction2,
        TransactionError as TransactionError2,
    },
};
use starcoin_vm_types::access_path::AccessPath;
use starcoin_vm_types::account_config::AccountResource;
use starcoin_vm_types::move_resource::MoveResource;
use starcoin_vm_types::transaction::SignedUserTransaction;
use starcoin_vm_types::transaction::{SignatureCheckedTransaction, Transaction, TransactionError};
use std::fmt::Display;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MultiAddress {
    VM1(AccountAddress),
    VM2(AccountAddress2),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MultiTransaction {
    VM1(Transaction),
    VM2(Transaction2),
}

impl From<AccountAddress> for MultiAddress {
    fn from(value: AccountAddress) -> Self {
        Self::VM1(value)
    }
}

impl From<AccountAddress2> for MultiAddress {
    fn from(value: AccountAddress2) -> Self {
        Self::VM2(value)
    }
}

impl MultiTransaction {
    pub fn sender_address(&self) -> MultiAddress {
        match self {
            MultiTransaction::VM1(txn) => match txn {
                Transaction::UserTransaction(txn) => MultiAddress::VM1(txn.sender()),
                Transaction::BlockMetadata(txn) => MultiAddress::VM1(txn.author()),
            },
            MultiTransaction::VM2(txn) => match txn {
                Transaction2::UserTransaction(txn) => MultiAddress::VM2(txn.sender()),
                Transaction2::BlockMetadata(txn) => MultiAddress::VM2(txn.author()),
                Transaction2::BlockEpilogue(txn, _) => MultiAddress::VM2(txn.author()),
            },
        }
    }

    pub fn access_path(&self) -> MultiAccessPath {
        match self.sender_address() {
            MultiAddress::VM1(addr) => {
                println!("jacktest: vm1,access_path: {}", addr);
                MultiAccessPath::VM1(AccessPath::resource_access_path(
                    addr,
                    AccountResource::struct_tag(),
                ))
            }
            MultiAddress::VM2(addr) => {
                println!("jacktest: vm2,access_path: {}", addr);
                MultiAccessPath::VM2(AccessPath2::resource_access_path(
                    addr,
                    AccountResource2::struct_tag(),
                ))
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub enum MultiSignedUserTransaction {
    VM1(SignedUserTransaction),
    VM2(SignedUserTransaction2),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum MultiSignatureCheckedTransaction {
    VM1(SignatureCheckedTransaction),
    VM2(SignatureCheckedTransaction2),
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

impl TryFrom<StcTransaction> for MultiSignedUserTransaction {
    type Error = Error;
    fn try_from(txn: StcTransaction) -> Result<Self, Self::Error> {
        Ok(match txn {
            StcTransaction::V1(txn) => MultiSignedUserTransaction::VM1(txn.try_into()?),
            StcTransaction::V2(txn) => MultiSignedUserTransaction::VM2(txn.try_into()?),
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MultiAccountAddress {
    VM1(AccountAddress),
    VM2(AccountAddress2),
}

impl MultiAccountAddress {
    pub fn to_hex(&self) -> String {
        match self {
            MultiAccountAddress::VM1(addr) => addr.to_hex(),
            MultiAccountAddress::VM2(addr) => addr.to_hex(),
        }
    }
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

impl From<SignedUserTransaction2> for MultiSignedUserTransaction {
    fn from(txn: SignedUserTransaction2) -> Self {
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

    pub fn raw_txn_bytes_len(&self) -> usize {
        match self {
            Self::VM1(sign) => sign.raw_txn_bytes_len(),
            Self::VM2(sign) => sign.raw_txn_bytes_len(),
        }
    }

    pub fn mock() -> Self {
        Self::VM1(SignedUserTransaction::mock())
    }

    #[inline]
    pub fn is_v1(&self) -> bool {
        matches!(self, Self::VM1(_))
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

impl TryFrom<Transaction2> for MultiSignedUserTransaction {
    type Error = Error;

    fn try_from(txn: Transaction2) -> Result<Self, Self::Error> {
        match txn {
            Transaction2::UserTransaction(txn) => Ok(Self::VM2(txn)),
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

impl From<MultiSignedUserTransaction> for Transaction2 {
    fn from(txn: MultiSignedUserTransaction) -> Self {
        match txn {
            MultiSignedUserTransaction::VM2(txn) => Transaction2::UserTransaction(txn),
            _ => panic!("Not a vm2 transaction."),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct APIInterruptedError(pub String);

impl std::fmt::Display for APIInterruptedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "API interrupted: {}", self.0)
    }
}

impl std::error::Error for APIInterruptedError {}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum MultiTransactionError {
    VM1(TransactionError),
    VM2(TransactionError2),
    APIInterrupted(APIInterruptedError),
}

impl Display for MultiTransactionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MultiTransactionError::VM1(e) => write!(f, "VM1 error: {}", e),
            MultiTransactionError::VM2(e) => write!(f, "VM2 error: {}", e),
            MultiTransactionError::APIInterrupted(msg) => write!(f, "API interrupted: {}", msg),
        }
    }
}

impl From<TransactionError> for MultiTransactionError {
    fn from(error: TransactionError) -> Self {
        Self::VM1(error)
    }
}

impl From<TransactionError2> for MultiTransactionError {
    fn from(error: TransactionError2) -> Self {
        Self::VM2(error)
    }
}

impl std::error::Error for MultiTransactionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::VM1(e) => Some(e),
            Self::VM2(e) => Some(e),
            Self::APIInterrupted(msg) => Some(msg),
        }
    }
}
