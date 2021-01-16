// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod client;
pub(crate) mod listener;
pub(crate) mod local_transactions;
pub(crate) mod queue;
pub(crate) mod ready;
pub(crate) mod replace;
pub(crate) mod scoring;
pub(crate) mod verifier;

pub use client::{AccountSeqNumberClient, Client};
use crypto::hash::HashValue;
pub use queue::{Status, TransactionQueue};
use std::ops::Deref;
use transaction_pool as tx_pool;
use types::{account_address::AccountAddress, transaction};
pub use verifier::Options as VerifierOptions;

pub type SeqNumber = u64;
//TODO gas and gas price should use MoveVM types.
pub type GasPrice = u64;
pub type Gas = u64;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct UnverifiedUserTransaction {
    txn: transaction::SignedUserTransaction,
}

impl UnverifiedUserTransaction {
    pub fn txn(&self) -> &transaction::SignedUserTransaction {
        &self.txn
    }

    pub fn hash(&self) -> HashValue {
        self.txn.id()
    }
}

impl From<UnverifiedUserTransaction> for transaction::SignedUserTransaction {
    fn from(txn: UnverifiedUserTransaction) -> transaction::SignedUserTransaction {
        txn.txn
    }
}

impl From<transaction::SignedUserTransaction> for UnverifiedUserTransaction {
    fn from(user_txn: transaction::SignedUserTransaction) -> Self {
        UnverifiedUserTransaction { txn: user_txn }
    }
}

impl Deref for UnverifiedUserTransaction {
    type Target = transaction::SignedUserTransaction;

    fn deref(&self) -> &Self::Target {
        &self.txn
    }
}

/// Transaction priority.
#[derive(Debug, PartialEq, Eq, PartialOrd, Clone, Copy)]
pub enum Priority {
    /// Regular transactions received over the network. (no priority boost)
    Regular,
    /// Transactions from retracted blocks (medium priority)
    ///
    /// When block becomes non-canonical we re-import the transactions it contains
    /// to the queue and boost their priority.
    Retracted,
    /// Local transactions (high priority)
    ///
    /// Transactions either from a local account or
    /// submitted over local RPC connection
    Local,
}

impl Priority {
    fn is_local(self) -> bool {
        matches!(self, Priority::Local)
    }
}
/// Transaction to verify.
#[derive(Clone)]
pub enum PoolTransaction {
    /// Fresh, never verified transaction.
    ///
    /// We need to do full verification of such transactions
    Unverified(UnverifiedUserTransaction),

    /// Transaction from retracted block.
    ///
    /// We could skip some parts of verification of such transactions
    Retracted(UnverifiedUserTransaction),

    /// Locally signed or retracted transaction.
    ///
    /// We can skip consistency verifications and just verify readiness.
    Local(transaction::PendingTransaction),
}

impl PoolTransaction {
    /// Return transaction hash
    pub fn hash(&self) -> HashValue {
        match self {
            PoolTransaction::Unverified(ref tx) => tx.hash(),
            PoolTransaction::Retracted(ref tx) => tx.hash(),
            PoolTransaction::Local(ref tx) => tx.id(),
        }
    }

    pub fn signed(&self) -> &transaction::SignedUserTransaction {
        match self {
            PoolTransaction::Unverified(t) => t.txn(),
            PoolTransaction::Retracted(t) => t.txn(),
            PoolTransaction::Local(t) => &t.transaction,
        }
    }

    /// Return transaction gas price
    pub fn gas_price(&self) -> GasPrice {
        match self {
            PoolTransaction::Unverified(ref tx) => tx.gas_unit_price(),
            PoolTransaction::Retracted(ref tx) => tx.gas_unit_price(),
            PoolTransaction::Local(ref tx) => tx.gas_unit_price(),
        }
    }

    fn gas(&self) -> Gas {
        match self {
            PoolTransaction::Unverified(ref tx) => tx.max_gas_amount(),
            PoolTransaction::Retracted(ref tx) => tx.max_gas_amount(),
            PoolTransaction::Local(ref tx) => tx.max_gas_amount(),
        }
    }

    fn transaction(&self) -> &transaction::RawUserTransaction {
        match self {
            PoolTransaction::Unverified(ref tx) => tx.raw_txn(),
            PoolTransaction::Retracted(ref tx) => tx.raw_txn(),
            PoolTransaction::Local(ref tx) => tx.raw_txn(),
        }
    }

    fn is_local(&self) -> bool {
        matches!(self, PoolTransaction::Local(..))
    }

    fn is_retracted(&self) -> bool {
        matches!(self, PoolTransaction::Retracted(..))
    }
}

/// Verified transaction stored in the pool.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedTransaction {
    transaction: transaction::PendingTransaction,
    // TODO: use transaction's hash/sender
    hash: HashValue,
    sender: AccountAddress,
    priority: Priority,
    insertion_id: usize,
}

impl VerifiedTransaction {
    /// Create `VerifiedTransaction` directly from `SignedUserTransaction`.
    ///
    /// This method should be used only:
    /// 1. for tests
    /// 2. In case we are converting pending block transactions that are already in the queue to match the function signature.
    pub fn from_pending_block_transaction(tx: transaction::SignedUserTransaction) -> Self {
        let hash = tx.id();
        let sender = tx.sender();
        VerifiedTransaction {
            transaction: tx.into(),
            hash,
            sender,
            priority: Priority::Retracted,
            insertion_id: 0,
        }
    }

    /// Gets transaction insertion id.
    pub(crate) fn insertion_id(&self) -> usize {
        self.insertion_id
    }

    /// Gets wrapped `SignedTransaction`
    pub fn signed(&self) -> &transaction::SignedUserTransaction {
        &self.transaction
    }

    /// Gets wrapped `PendingTransaction`
    pub fn pending(&self) -> &transaction::PendingTransaction {
        &self.transaction
    }
}

impl tx_pool::VerifiedTransaction for VerifiedTransaction {
    type Hash = HashValue;
    type Sender = AccountAddress;

    fn hash(&self) -> &Self::Hash {
        &self.hash
    }

    fn mem_usage(&self) -> usize {
        self.transaction.raw_txn_bytes_len()
    }

    fn sender(&self) -> &Self::Sender {
        &self.sender
    }
}

/// Scoring properties for verified transaction.
pub trait ScoredTransaction {
    /// Gets transaction priority.
    fn priority(&self) -> Priority;

    /// Gets transaction gas price.
    fn gas_price(&self) -> u64;

    /// Gets transaction seq number.
    fn seq_number(&self) -> u64;
}

impl ScoredTransaction for VerifiedTransaction {
    fn priority(&self) -> Priority {
        self.priority
    }

    /// Gets transaction gas price.
    fn gas_price(&self) -> u64 {
        self.transaction.gas_unit_price()
    }

    /// Gets transaction nonce.
    fn seq_number(&self) -> SeqNumber {
        self.transaction.sequence_number()
    }
}

/// How to prioritize transactions in the pool
///
/// TODO: Implement more strategies.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PrioritizationStrategy {
    /// Simple gas-price based prioritization.
    GasPriceOnly,
}

/// Transaction ordering when requesting pending set.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PendingOrdering {
    /// Get pending transactions ordered by their priority (potentially expensive)
    Priority,
    /// Get pending transactions without any care of particular ordering (cheaper).
    Unordered,
}

/// Pending set query settings
#[derive(Debug, Clone)]
pub struct PendingSettings {
    /// Current block number (affects readiness of some transactions).
    pub block_number: u64,
    /// Current timestamp (affects readiness of some transactions).
    pub current_timestamp: u64,
    /// Maximal number of transactions in pending the set.
    pub max_len: usize,
    /// Ordering of transactions.
    pub ordering: PendingOrdering,
}

impl PendingSettings {
    /// Get all transactions (no cap or len limit) prioritized.
    pub fn all_prioritized(block_number: u64, current_timestamp: u64) -> Self {
        PendingSettings {
            block_number,
            current_timestamp,
            max_len: usize::max_value(),
            ordering: PendingOrdering::Priority,
        }
    }
}

pub type TxStatus = types::transaction::TxStatus;
