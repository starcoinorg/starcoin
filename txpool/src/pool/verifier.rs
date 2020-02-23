//! Transaction Verifier
//!
//! Responsible for verifying a transaction before importing to the pool.
//! Should make sure that the transaction is structuraly valid.
//!
//! May have some overlap with `Readiness` since we don't want to keep around
//! stalled transactions.
use std::{
    cmp,
    sync::{
        atomic::{self, AtomicUsize},
        Arc,
    },
};

use common_crypto::hash::*;
use tx_pool;
use types::transaction;

use super::{client::Client, Gas, GasPrice, VerifiedTransaction};
use crate::pool::scoring;
use std::ops::Deref;

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

/// Transaction to verify.
#[cfg_attr(test, derive(Clone))]
pub enum Transaction {
    /// Fresh, never verified transaction.
    ///
    /// We need to do full verification of such transactions
    Unverified(transaction::SignedUserTransaction),

    /// Transaction from retracted block.
    ///
    /// We could skip some parts of verification of such transactions
    Retracted(transaction::SignedUserTransaction),

    /// Locally signed or retracted transaction.
    ///
    /// We can skip consistency verifications and just verify readiness.
    Local(transaction::PendingTransaction),
}

impl Transaction {
    /// Return transaction hash
    pub fn hash(&self) -> HashValue {
        match *self {
            Transaction::Unverified(ref tx) => CryptoHash::crypto_hash(&tx),
            Transaction::Retracted(ref tx) => CryptoHash::crypto_hash(&tx),
            Transaction::Local(ref tx) => CryptoHash::crypto_hash(tx.deref()),
        }
    }

    pub fn signed(&self) -> &transaction::SignedUserTransaction {
        match self {
            Transaction::Unverified(t) => t,
            Transaction::Retracted(t) => t,
            Transaction::Local(t) => &t.transaction,
        }
    }

    /// Return transaction gas price
    pub fn gas_price(&self) -> GasPrice {
        match self {
            Transaction::Unverified(ref tx) => tx.gas_unit_price(),
            Transaction::Retracted(ref tx) => tx.gas_unit_price(),
            Transaction::Local(ref tx) => tx.gas_unit_price(),
        }
    }

    fn gas(&self) -> Gas {
        match self {
            Transaction::Unverified(ref tx) => tx.max_gas_amount(),
            Transaction::Retracted(ref tx) => tx.max_gas_amount(),
            Transaction::Local(ref tx) => tx.max_gas_amount(),
        }
    }

    fn transaction(&self) -> &transaction::RawUserTransaction {
        match self {
            Transaction::Unverified(ref tx) => tx.raw_txn(),
            Transaction::Retracted(ref tx) => tx.raw_txn(),
            Transaction::Local(ref tx) => tx.raw_txn(),
        }
    }

    fn is_local(&self) -> bool {
        match self {
            Transaction::Local(..) => true,
            _ => false,
        }
    }

    fn is_retracted(&self) -> bool {
        match self {
            Transaction::Retracted(..) => true,
            _ => false,
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

impl<C: Client> tx_pool::Verifier<Transaction>
    for Verifier<C, scoring::SeqNumberAndGasPrice, VerifiedTransaction>
{
    type Error = transaction::TransactionError;
    type VerifiedTransaction = VerifiedTransaction;

    fn verify_transaction(
        &self,
        tx: Transaction,
    ) -> Result<Self::VerifiedTransaction, Self::Error> {
        // TODO: implement transaction verify
        match tx {
            Transaction::Unverified(t) => {
                Ok(VerifiedTransaction::from_pending_block_transaction(t))
            }
            Transaction::Retracted(t) => Ok(VerifiedTransaction::from_pending_block_transaction(t)),
            Transaction::Local(pt) => Ok(VerifiedTransaction::from_pending_block_transaction(
                pt.transaction,
            )),
        }
    }
}
