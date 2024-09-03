// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod adapter;

pub(crate) use adapter::{PublishModuleBundleOption, SessionAdapter};
use starcoin_vm_types::block_metadata::BlockMetadata;
use starcoin_vm_types::transaction::{SignedUserTransaction, Transaction};

#[derive(Debug)]
pub enum PreprocessedTransaction {
    UserTransaction(Box<SignedUserTransaction>),
    BlockMetadata(BlockMetadata),
}

#[inline]
pub fn preprocess_transaction(txn: Transaction) -> crate::vm_adapter::PreprocessedTransaction {
    match txn {
        Transaction::BlockMetadata(b) => {
            crate::vm_adapter::PreprocessedTransaction::BlockMetadata(b)
        }
        Transaction::UserTransaction(txn) => {
            crate::vm_adapter::PreprocessedTransaction::UserTransaction(Box::new(txn))
        }
    }
}
