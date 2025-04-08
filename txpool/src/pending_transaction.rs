use starcoin_types::multi_transaction::MultiSignedUserTransaction;
use std::ops::Deref;

type BlockNumber = u64;

/// Transaction activation condition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Condition {
    /// Valid at this block number or later.
    Number(BlockNumber),
    /// Valid at this unix time or later.
    Timestamp(u64),
}

/// Queued transaction with additional information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingTransaction {
    /// Signed transaction data.
    pub transaction: MultiSignedUserTransaction,
    /// To be activated at this condition. `None` for immediately.
    pub condition: Option<Condition>,
}

impl PendingTransaction {
    /// Create a new pending transaction from signed transaction.
    pub fn new(signed: MultiSignedUserTransaction, condition: Option<Condition>) -> Self {
        PendingTransaction {
            transaction: signed,
            condition,
        }
    }
}

impl Deref for PendingTransaction {
    type Target = MultiSignedUserTransaction;

    fn deref(&self) -> &Self::Target {
        &self.transaction
    }
}

impl From<MultiSignedUserTransaction> for PendingTransaction {
    fn from(t: MultiSignedUserTransaction) -> Self {
        PendingTransaction {
            transaction: t,
            condition: None,
        }
    }
}
