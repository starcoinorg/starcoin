// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::view::signed_user_transaction_view::SignedUserTransactionView;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;
use starcoin_vm_types::transaction::SignedUserTransaction;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum BlockTransactionsView {
    Hashes(Vec<HashValue>),
    Full(Vec<SignedUserTransactionView>),
}

impl BlockTransactionsView {
    pub fn txn_hashes(&self) -> Vec<HashValue> {
        match self {
            Self::Hashes(h) => h.clone(),
            Self::Full(f) => f.iter().map(|t| t.transaction_hash).collect(),
        }
    }
}

impl TryFrom<Vec<SignedUserTransaction>> for BlockTransactionsView {
    type Error = anyhow::Error;

    fn try_from(txns: Vec<SignedUserTransaction>) -> Result<Self, Self::Error> {
        Ok(Self::Full(
            txns.into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
        ))
    }
}

impl TryFrom<BlockTransactionsView> for Vec<SignedUserTransaction> {
    type Error = anyhow::Error;

    fn try_from(tx_view: BlockTransactionsView) -> Result<Self, Self::Error> {
        match tx_view {
            BlockTransactionsView::Full(full) => Ok(full
                .into_iter()
                .map(|transaction_view| {
                    SignedUserTransaction::new(
                        transaction_view.raw_txn.into(),
                        transaction_view.authenticator,
                    )
                })
                .collect()),
            _ => Err(anyhow::Error::msg("not support")),
        }
    }
}

impl From<Vec<HashValue>> for BlockTransactionsView {
    fn from(txns: Vec<HashValue>) -> Self {
        Self::Hashes(txns)
    }
}
