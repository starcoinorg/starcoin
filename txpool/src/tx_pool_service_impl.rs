// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::pool::{verifier, PrioritizationStrategy, TransactionQueue, VerifierOptions};
use crate::{
    pool,
    pool::{PendingOrdering, PendingSettings},
};
use anyhow::Result;
use std::sync::Arc;
use types::{
    transaction,
    transaction::{SignatureCheckedTransaction, SignedUserTransaction, UnverifiedUserTransaction},
};

pub struct TxPoolServiceImpl {
    queue: TransactionQueue,
}

impl TxPoolServiceImpl {
    pub fn new(pool_options: tx_pool::Options, verifier_options: VerifierOptions) -> Self {
        Self {
            queue: TransactionQueue::new(
                pool_options,
                verifier_options,
                PrioritizationStrategy::GasPriceOnly,
            ),
        }
    }
}

impl super::TxPoolService for TxPoolServiceImpl {
    fn import_txns<C>(
        &self,
        client: C,
        txns: Vec<transaction::UnverifiedUserTransaction>,
    ) -> Vec<Result<(), transaction::TransactionError>>
    where
        C: pool::NonceClient + pool::Client + Clone,
    {
        let txns = txns
            .into_iter()
            .map(|t| verifier::Transaction::Unverified(t.into_inner()));
        self.queue.import(client, txns)
    }

    fn get_pending_txns<C>(
        &self,
        client: C,
        best_block_number: u64,
        best_block_timestamp: u64,
        max_len: u64,
    ) -> Option<Vec<Arc<pool::VerifiedTransaction>>>
    where
        C: pool::NonceClient,
    {
        let pendings = self.queue.pending(
            client,
            PendingSettings {
                nonce_cap: None,
                max_len: max_len as usize,
                ordering: PendingOrdering::Priority,
                block_number: best_block_number,
                current_timestamp: best_block_timestamp,
            },
        );
        if pendings.is_empty() {
            None
        } else {
            Some(pendings)
        }
    }
}
