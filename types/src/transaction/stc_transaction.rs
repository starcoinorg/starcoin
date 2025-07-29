// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;
pub use starcoin_vm2_vm_types::transaction::Transaction as Transaction2;

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
