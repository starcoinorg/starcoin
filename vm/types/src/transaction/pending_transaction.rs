use super::SignedUserTransactionV2;
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
    pub transaction: SignedUserTransactionV2,
    /// To be activated at this condition. `None` for immediately.
    pub condition: Option<Condition>,
}

impl PendingTransaction {
    /// Create a new pending transaction from signed transaction.
    pub fn new(signed: SignedUserTransactionV2, condition: Option<Condition>) -> Self {
        PendingTransaction {
            transaction: signed,
            condition,
        }
    }
}

impl Deref for PendingTransaction {
    type Target = SignedUserTransactionV2;

    fn deref(&self) -> &Self::Target {
        &self.transaction
    }
}

impl From<SignedUserTransactionV2> for PendingTransaction {
    fn from(t: SignedUserTransactionV2) -> Self {
        PendingTransaction {
            transaction: t,
            condition: None,
        }
    }
}
