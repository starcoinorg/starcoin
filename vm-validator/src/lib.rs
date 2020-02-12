// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0
use starcoin_types::{transaction::SignedUserTransaction, vm_error::VMStatus};
use vm_runtime::VMVerifier;

#[async_trait::async_trait]
pub trait TransactionValidation: Send + Sync + Clone {
    type ValidationInstance: VMVerifier;
    /// Validate a txn from client
    async fn validate_transaction(&self, _txn: SignedUserTransaction) -> Result<Option<VMStatus>>;
}
