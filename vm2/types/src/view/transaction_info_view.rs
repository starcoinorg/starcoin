// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::view::{str_view::StrView, transaction_status_view::TransactionStatusView};
use anyhow::bail;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;
use starcoin_vm_types::transaction::{RichTransactionInfo, TransactionInfo, TransactionStatus};

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct TransactionInfoView {
    pub block_hash: HashValue,
    pub block_number: StrView<u64>,
    /// The hash of this transaction.
    pub transaction_hash: HashValue,
    /// The index of this transaction in block
    pub transaction_index: u32,
    /// The index of this transaction in chain
    pub transaction_global_index: StrView<u64>,
    /// The root hash of Sparse Merkle Tree describing the world state at the end of this
    /// transaction.
    pub state_root_hash: HashValue,

    /// The root hash of Merkle Accumulator storing all events emitted during this transaction.
    pub event_root_hash: HashValue,

    /// The amount of gas used.
    pub gas_used: StrView<u64>,

    /// The vm status. If it is not `Executed`, this will provide the general error class. Execution
    /// failures and Move abort's receive more detailed information. But other errors are generally
    /// categorized with no status code or other information
    pub status: TransactionStatusView,
}

impl TransactionInfoView {
    pub fn new(txn_info: RichTransactionInfo) -> Self {
        Self {
            block_hash: txn_info.block_id,
            block_number: txn_info.block_number.into(),
            transaction_hash: txn_info.transaction_hash,
            transaction_index: txn_info.transaction_index,
            transaction_global_index: txn_info.transaction_global_index.into(),
            state_root_hash: txn_info.transaction_info.state_root_hash,
            event_root_hash: txn_info.transaction_info.event_root_hash,
            gas_used: txn_info.transaction_info.gas_used.into(),
            status: TransactionStatusView::from(txn_info.transaction_info.status),
        }
    }
}

impl From<RichTransactionInfo> for TransactionInfoView {
    fn from(txn_info: RichTransactionInfo) -> Self {
        Self::new(txn_info)
    }
}

impl TryFrom<TransactionInfoView> for RichTransactionInfo {
    type Error = anyhow::Error;

    fn try_from(view: TransactionInfoView) -> Result<Self, Self::Error> {
        let status: TransactionStatus = view.status.clone().into();
        match status {
            TransactionStatus::Keep(kept_status) => Ok(Self::new(
                view.block_hash,
                view.block_number.0,
                TransactionInfo {
                    transaction_hash: view.transaction_hash,

                    state_root_hash: view.state_root_hash,
                    event_root_hash: view.event_root_hash,
                    gas_used: view.gas_used.0,
                    status: kept_status,
                },
                view.transaction_index,
                view.transaction_global_index.0,
            )),
            TransactionStatus::Discard(_) => {
                bail!("TransactionInfoView's status is discard, {:?}, can not convert to RichTransactionInfo", view);
            }
            TransactionStatus::Retry => {
                bail!("TransactionInfoView's status is Retry, {:?}, can not convert to RichTransactionInfo", view);
            }
        }
    }
}
