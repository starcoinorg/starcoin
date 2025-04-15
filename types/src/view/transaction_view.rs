// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    block::{Block, BlockNumber},
    view::{
        block_metadata_view::BlockMetadataView,
        signed_user_transaction_view::SignedUserTransactionView, str_view::StrView
    }
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;
use starcoin_vm_types::transaction::Transaction;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct TransactionView {
    pub block_hash: HashValue,
    pub block_number: StrView<BlockNumber>,
    pub transaction_hash: HashValue,
    pub transaction_index: u32,
    pub block_metadata: Option<BlockMetadataView>,
    pub user_transaction: Option<SignedUserTransactionView>,
}

impl TransactionView {
    pub fn new(txn: Transaction, block: &Block) -> anyhow::Result<Self> {
        let transaction_hash = txn.id();
        let block_hash = block.id();
        let block_number = block.header.number();
        let transaction_index = match &txn {
            Transaction::BlockMetadata(_) => 0,
            _ => {
                1 + block
                    .transactions()
                    .iter()
                    .position(|t| t.id() == transaction_hash)
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "cannot find txn {} in block {}",
                            transaction_hash,
                            block_hash
                        )
                    })? as u32
            }
        };

        let (meta, txn) = match txn {
            Transaction::BlockMetadata(meta) => (Some(meta.into()), None),
            Transaction::UserTransaction(t) => (None, Some(t.try_into()?)),
        };
        Ok(Self {
            block_hash,
            block_number: block_number.into(),
            transaction_hash,
            transaction_index,
            block_metadata: meta,
            user_transaction: txn,
        })
    }
}
