// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;
pub use starcoin_vm2_vm_types::transaction::Transaction as Transaction2;

use crate::multi_transaction::{MultiAccountAddress, MultiTransaction};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum StcTransaction {
    V1(super::Transaction),
    V2(Transaction2),
}

impl StcTransaction {
    pub fn id(&self) -> HashValue {
        match self {
            StcTransaction::V1(txn) => txn.id(),
            StcTransaction::V2(txn) => txn.id(),
        }
    }

    pub fn to_v1(self) -> Option<super::Transaction> {
        match self {
            StcTransaction::V1(txn) => Some(txn),
            StcTransaction::V2(_) => None,
        }
    }

    pub fn to_v2(self) -> Option<Transaction2> {
        match self {
            StcTransaction::V1(_) => None,
            StcTransaction::V2(txn) => Some(txn),
        }
    }

    pub fn to_transaction(self) -> MultiTransaction {
        match self {
            StcTransaction::V1(txn) => MultiTransaction::VM1(txn),
            StcTransaction::V2(txn) => MultiTransaction::VM2(txn),
        }
    }

    pub fn address(&self) -> MultiAccountAddress {
        match self {
            StcTransaction::V1(txn) => match txn {
                super::Transaction::UserTransaction(txn) => MultiAccountAddress::VM1(txn.sender()),
                super::Transaction::BlockMetadata(txn) => MultiAccountAddress::VM1(txn.author()),
            },
            StcTransaction::V2(txn) => match txn {
                Transaction2::UserTransaction(txn) => MultiAccountAddress::VM2(txn.sender()),
                Transaction2::BlockMetadata(txn) => MultiAccountAddress::VM2(txn.author()),
                Transaction2::BlockEpilogue(txn) => MultiAccountAddress::VM2(txn.author()),
            },
        }
    }
}

impl From<super::Transaction> for StcTransaction {
    fn from(txn: super::Transaction) -> Self {
        StcTransaction::V1(txn)
    }
}

impl From<Transaction2> for StcTransaction {
    fn from(txn: Transaction2) -> Self {
        StcTransaction::V2(txn)
    }
}
