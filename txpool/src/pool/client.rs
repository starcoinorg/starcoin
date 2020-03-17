// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Transaction Pool state client.
//!
//! `Client` encapsulates all external data required for the verifaction and readiness.
//! It includes any Ethereum state parts required for checking the transaction and
//! any consensus-required structure of the transaction.

use super::{SeqNumber, UnverifiedUserTransaction};
use std::{any::Any, fmt};
use types::{account_address::AccountAddress as Address, transaction};

/// State sequence number client
pub trait AccountSeqNumberClient: fmt::Debug + Clone + Any {
    /// Fetch only account nonce for given sender.
    fn account_seq_number(&self, address: &Address) -> SeqNumber;
}

/// Verification client.
pub trait Client: fmt::Debug {
    // /// Perform basic/cheap transaction verification.
    // ///
    // /// This should include all cheap checks that can be done before
    // /// actually checking the signature, like chain-replay protection.
    // ///
    // /// This method is currently used only for verifying local transactions.
    // fn verify_transaction_basic(
    //     &self,
    //     t: &UnverifiedUserTransaction,
    // ) -> Result<(), transaction::TransactionError>;

    /// Structurally verify given transaction.
    fn verify_transaction(
        &self,
        tx: UnverifiedUserTransaction,
    ) -> Result<transaction::SignatureCheckedTransaction, transaction::TransactionError>;
}
