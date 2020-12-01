use crate::vm_status::VMStatus;
use std::{error, fmt};

type Gas = u64;
type GasPrice = u64;

#[derive(Debug, PartialEq, Clone)]
/// Errors concerning transaction processing.
pub enum Error {
    /// Transaction is already imported to the queue
    AlreadyImported,
    /// Transaction is not valid anymore (state already has higher nonce)
    Old,
    /// Transaction was not imported to the queue because limit has been reached.
    LimitReached,
    /// Transaction's gas price is below threshold.
    InsufficientGasPrice {
        /// Minimal expected gas price
        minimal: GasPrice,
        /// Transaction gas price
        got: GasPrice,
    },
    /// Transaction has too low fee
    /// (there is already a transaction with the same sender-nonce but higher gas price)
    TooCheapToReplace {
        /// previous transaction's gas price
        prev: Option<GasPrice>,
        /// new transaction's gas price
        new: Option<GasPrice>,
    },
    /// Transaction's gas is below currently set minimal gas requirement.
    InsufficientGas {
        /// Minimal expected gas
        minimal: Gas,
        /// Transaction gas
        got: Gas,
    },
    /// Sender doesn't have enough funds to pay for this transaction
    InsufficientBalance {
        /// Senders balance
        balance: u64,
        /// Transaction cost
        cost: u64,
    },
    /// Transactions gas is higher then current gas limit
    GasLimitExceeded {
        /// Current gas limit
        limit: Gas,
        /// Declared transaction gas
        got: Gas,
    },
    /// Transaction's gas limit (aka gas) is invalid.
    //    InvalidGasLimit(OutOfBounds<Gas>),
    /// Transaction sender is banned.
    SenderBanned,
    /// Transaction receipient is banned.
    RecipientBanned,
    /// Contract creation code is banned.
    CodeBanned,
    /// Invalid chain ID given.
    InvalidChainId,
    /// Not enough permissions given by permission contract.
    NotAllowed,
    /// Signature error
    InvalidSignature(String),
    /// Transaction too big
    TooBig,
    CallErr(CallError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Error::*;
        let msg = match self {
            AlreadyImported => "Already imported".into(),
            Old => "No longer valid".into(),
            TooCheapToReplace { prev, new } => format!(
                "Gas price too low to replace, previous tx gas: {:?}, new tx gas: {:?}",
                prev, new
            ),
            LimitReached => "Transaction limit reached".into(),
            InsufficientGasPrice { minimal, got } => {
                format!("Insufficient gas price. Min={}, Given={}", minimal, got)
            }
            InsufficientGas { minimal, got } => {
                format!("Insufficient gas. Min={}, Given={}", minimal, got)
            }
            InsufficientBalance { balance, cost } => format!(
                "Insufficient balance for transaction. Balance={}, Cost={}",
                balance, cost
            ),
            GasLimitExceeded { limit, got } => {
                format!("Gas limit exceeded. Limit={}, Given={}", limit, got)
            }
            //            InvalidGasLimit(ref err) => format!("Invalid gas limit. {}", err),
            SenderBanned => "Sender is temporarily banned.".into(),
            RecipientBanned => "Recipient is temporarily banned.".into(),
            CodeBanned => "Contract code is temporarily banned.".into(),
            InvalidChainId => "Transaction of this chain ID is not allowed on this chain.".into(),
            InvalidSignature(ref err) => format!("Transaction has invalid signature: {}.", err),
            NotAllowed => {
                "Sender does not have permissions to execute this type of transaction".into()
            }
            TooBig => "Transaction too big".into(),
            CallErr(call_err) => format!("Call txn err: {}.", call_err),
        };

        f.write_fmt(format_args!("Transaction error ({})", msg))
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        "Transaction error"
    }
}

/// Result of executing the transaction.
#[derive(PartialEq, Debug, Clone)]
pub enum CallError {
    /// Couldn't find the transaction in the chain.
    TransactionNotFound,
    /// Couldn't find requested block's state in the chain.
    StatePruned,
    /// Couldn't find an amount of gas that didn't result in an exception.
    //    Exceptional(VMStatus),

    /// Corrupt state.
    StateCorrupt,
    /// Error executing.
    ExecutionError(VMStatus),
}

impl From<VMStatus> for CallError {
    fn from(error: VMStatus) -> Self {
        CallError::ExecutionError(error)
    }
}

impl fmt::Display for CallError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::CallError::*;
        let msg = match *self {
            TransactionNotFound => "Transaction couldn't be found in the chain".into(),
            StatePruned => "Couldn't find the transaction block's state in the chain".into(),
            //            Exceptional(ref e) => format!("An exception ({}) happened in the execution", e),
            StateCorrupt => "Stored state found to be corrupted.".into(),
            ExecutionError(ref e) => format!("Execution error: {}", e),
        };

        f.write_fmt(format_args!("Transaction execution error ({}).", msg))
    }
}
