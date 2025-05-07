// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use schemars::JsonSchema;
use crate::types::RawUserTransactionView;
use serde::{Deserialize, Serialize};
use starcoin_types::multi_transaction::MultiRawUserTransaction;
use starcoin_vm2_types::view::raw_user_transaction_view::RawUserTransactionView as RawUserTransactionViewV2;

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub enum MultiRawUserTransactionView {
    VM1(RawUserTransactionView),
    VM2(RawUserTransactionViewV2),
}

impl TryFrom<MultiRawUserTransaction> for MultiRawUserTransactionView {
    type Error = anyhow::Error;

    fn try_from(origin: MultiRawUserTransaction) -> Result<Self, Self::Error> {
        origin.try_into()
    }
}

impl TryFrom<MultiRawUserTransactionView> for MultiRawUserTransaction {
    type Error = ();

    fn try_from(transaction_view: MultiRawUserTransactionView) -> Result<Self, Self::Error> {
        let multi_txn = match transaction_view {
            MultiRawUserTransactionView::VM1(txn) => MultiRawUserTransaction::VM1(txn.into()),
            MultiRawUserTransactionView::VM2(txn) => MultiRawUserTransaction::VM2(txn.into()),
        };
        Ok(multi_txn)
    }
}
