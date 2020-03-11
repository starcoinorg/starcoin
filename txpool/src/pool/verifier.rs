// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Transaction Verifier
//!
//! Responsible for verifying a transaction before importing to the pool.
//! Should make sure that the transaction is structuraly valid.
//!
//! May have some overlap with `Readiness` since we don't want to keep around
//! stalled transactions.
use crate::pool::{
    client::Client,
    scoring,
    Gas,
    GasPrice,
    PoolTransaction,
    Priority,
    UnverifiedUserTransaction,
    VerifiedTransaction,
};
use std::sync::{atomic::AtomicUsize, Arc};
use tx_pool;
use types::transaction;

/// Verification options.
#[derive(Debug, Clone, PartialEq)]
pub struct Options {
    /// Minimal allowed gas price.
    pub minimal_gas_price: GasPrice,
    /// Current block gas limit.
    pub block_gas_limit: Gas,
    /// Maximal gas limit for a single transaction.
    pub tx_gas_limit: Gas,
    /// Skip checks for early rejection, to make sure that local transactions are always imported.
    pub no_early_reject: bool,
}

#[cfg(test)]
impl Default for Options {
    fn default() -> Self {
        Options {
            minimal_gas_price: 0,
            block_gas_limit: Gas::max_value(),
            tx_gas_limit: Gas::max_value(),
            no_early_reject: false,
        }
    }
}

/// Transaction verifier.
///
/// Verification can be run in parallel for all incoming transactions.
#[derive(Debug)]
pub struct Verifier<C, S, V> {
    client: C,
    options: Options,
    id: Arc<AtomicUsize>,
    transaction_to_replace: Option<(S, Arc<V>)>,
}

impl<C, S, V> Verifier<C, S, V> {
    /// Creates new transaction verfier with specified options.
    pub fn new(
        client: C,
        options: Options,
        id: Arc<AtomicUsize>,
        transaction_to_replace: Option<(S, Arc<V>)>,
    ) -> Self {
        Verifier {
            client,
            options,
            id,
            transaction_to_replace,
        }
    }
}

impl<C: Client> tx_pool::Verifier<PoolTransaction>
    for Verifier<C, scoring::SeqNumberAndGasPrice, VerifiedTransaction>
{
    type Error = transaction::TransactionError;
    type VerifiedTransaction = VerifiedTransaction;

    fn verify_transaction(
        &self,
        tx: PoolTransaction,
    ) -> Result<Self::VerifiedTransaction, Self::Error> {
        let hash = tx.hash();
        let is_local_txn = tx.is_local();
        let is_retracted = tx.is_retracted();
        let verified_txn = match tx {
            PoolTransaction::Unverified(unverified) | PoolTransaction::Retracted(unverified) => {
                match self.client.verify_transaction(unverified) {
                    Ok(txn) => transaction::PendingTransaction::from(txn.into_inner()),
                    Err(err) => {
                        debug!(target: "txqueue", "[{:?}] Rejected tx {:?}", hash, err);
                        return Err(err);
                    }
                }
            }
            PoolTransaction::Local(txn) => {
                let user_txn = txn.transaction.clone();
                match self
                    .client
                    .verify_transaction(UnverifiedUserTransaction::from(user_txn))
                {
                    Ok(_) => txn,
                    Err(err) => {
                        warn!(target: "txqueue", "[{:?}] Rejected local tx {:?}", hash, err);
                        return Err(err);
                    }
                }
            }
        };

        let sender = verified_txn.sender();
        let priority = match (is_local_txn, is_retracted) {
            (true, _) => Priority::Local,
            (false, true) => Priority::Retracted,
            (false, false) => Priority::Local,
        };
        Ok(VerifiedTransaction {
            transaction: verified_txn,
            hash,
            sender,
            priority,
            insertion_id: self.id.fetch_add(1, std::sync::atomic::Ordering::AcqRel),
        })
    }
}
