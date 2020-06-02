use super::SignedUserTransaction;
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
    pub transaction: SignedUserTransaction,
    /// To be activated at this condition. `None` for immediately.
    pub condition: Option<Condition>,
}

impl PendingTransaction {
    /// Create a new pending transaction from signed transaction.
    pub fn new(signed: SignedUserTransaction, condition: Option<Condition>) -> Self {
        PendingTransaction {
            transaction: signed,
            condition,
        }
    }
}

impl Deref for PendingTransaction {
    type Target = SignedUserTransaction;

    fn deref(&self) -> &Self::Target {
        &self.transaction
    }
}

impl From<SignedUserTransaction> for PendingTransaction {
    fn from(t: SignedUserTransaction) -> Self {
        PendingTransaction {
            transaction: t,
            condition: None,
        }
    }
}
