//! Transaction Pool state client.
//!
//! `Client` encapsulates all external data required for the verifaction and readiness.
//! It includes any Ethereum state parts required for checking the transaction and
//! any consensus-required structure of the transaction.

use super::{Nonce, UnverifiedUserTransaction};
use crate::pool::Gas;
use common_crypto::hash::HashValue;
use std::fmt;
use types::{account_address::AccountAddress as Address, transaction};

/// State nonce client
#[async_trait]
pub trait NonceClient: fmt::Debug + Clone + Sync + Unpin + 'static {
    /// Fetch only account nonce for given sender.
    async fn account_nonce(&self, address: &Address) -> Nonce;
}

/// Verification client.
pub trait Client: fmt::Debug + Sync {
    /// Is transaction with given hash already in the blockchain?
    fn transaction_already_included(&self, hash: &HashValue) -> bool;

    /// Perform basic/cheap transaction verification.
    ///
    /// This should include all cheap checks that can be done before
    /// actually checking the signature, like chain-replay protection.
    ///
    /// This method is currently used only for verifying local transactions.
    fn verify_transaction_basic(
        &self,
        t: &UnverifiedUserTransaction,
    ) -> Result<(), transaction::TransactionError>;

    /// Structurally verify given transaction.
    fn verify_transaction(
        &self,
        tx: UnverifiedUserTransaction,
    ) -> Result<transaction::SignatureCheckedTransaction, transaction::TransactionError>;

    // /// Estimate minimal gas requirurement for given transaction.
    // fn required_gas(&self, tx: &transaction::RawUserTransaction) -> Gas;

    // /// Fetch account details for given sender.
    // fn account_details(&self, address: &Address) -> AccountDetails;
    //
    // /// Classify transaction (check if transaction is filtered by some contracts).
    // fn transaction_type(&self, tx: &transaction::SignedUserTransaction) -> TransactionType;
    //
    // /// Performs pre-validation of RLP decoded transaction
    // fn decode_transaction(
    //     &self,
    //     transaction: &[u8],
    // ) -> Result<transaction::UnverifiedTransaction, transaction::Error>;
}
