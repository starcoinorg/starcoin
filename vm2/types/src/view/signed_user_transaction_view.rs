// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::view::raw_user_transaction_view::RawUserTransactionView;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;
use starcoin_vm_types::transaction::{
    authenticator::TransactionAuthenticator, SignedUserTransaction,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct SignedUserTransactionView {
    pub transaction_hash: HashValue,
    /// The raw transaction
    pub raw_txn: RawUserTransactionView,
    /// Public key and signature to authenticate
    pub authenticator: TransactionAuthenticator,
}

impl TryFrom<SignedUserTransaction> for SignedUserTransactionView {
    type Error = anyhow::Error;

    fn try_from(txn: SignedUserTransaction) -> Result<Self, Self::Error> {
        let auth = txn.authenticator();
        let txn_hash = txn.id();
        Ok(Self {
            transaction_hash: txn_hash,
            raw_txn: txn.into_raw_transaction().try_into()?,
            authenticator: auth,
        })
    }
}
