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
use common_crypto::hash::{CryptoHash, HashValue};
use transaction_pool as tx_pool;
use types::{account_address::AccountAddress, transaction};

pub type SeqNumber = u64;
pub type GasPrice = u64;
pub type Gas = u64;
pub use client::{AccountSeqNumberClient, Client};
pub use queue::{Status, TransactionQueue};
pub use verifier::Options as VerifierOptions;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct UnverifiedUserTransaction(transaction::SignedUserTransaction);

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
    fn is_local(&self) -> bool {
        match *self {
            Priority::Local => true,
            _ => false,
        }
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
        let hash = CryptoHash::crypto_hash(&tx);
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
    /// Nonce cap (for dust protection; EIP-168)
    pub nonce_cap: Option<SeqNumber>,
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
            nonce_cap: None,
            max_len: usize::max_value(),
            ordering: PendingOrdering::Priority,
        }
    }
}

/// Pool transactions status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TxStatus {
    /// Added transaction
    Added,
    /// Rejected transaction
    Rejected,
    /// Dropped transaction
    Dropped,
    /// Invalid transaction
    Invalid,
    /// Canceled transaction
    Canceled,
    /// Culled transaction
    Culled,
}
