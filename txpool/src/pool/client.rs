// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Transaction Pool state client.
//!
//! `Client` encapsulates all external data required for the verifaction and readiness.
//! It includes any Ethereum state parts required for checking the transaction and
//! any consensus-required structure of the transaction.

use super::{SeqNumber, UnverifiedUserTransaction};
use common_crypto::hash::HashValue;
use std::fmt;
use types::{account_address::AccountAddress as Address, transaction};

/// State sequence number client
#[async_trait]
pub trait AccountSeqNumberClient: fmt::Debug + Clone + Sync + Unpin + 'static {
    /// Fetch only account nonce for given sender.
    async fn account_seq_number(&self, address: &Address) -> SeqNumber;
}

/// Verification client.
pub trait Client: fmt::Debug + Sync {
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
}
