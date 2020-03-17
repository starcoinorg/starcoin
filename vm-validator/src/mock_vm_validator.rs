// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::vm_validator::TransactionValidation;
use starcoin_state_view::StateView;
use starcoin_types::{
    transaction::SignedUserTransaction,
    vm_error::{StatusCode, VMStatus},
};

#[derive(Clone)]
pub struct MockVMValidator;

impl VMVerifier for MockVMValidator {
    fn validate_transaction(
        &self,
        _transaction: SignedUserTransaction,
        _state_view: &dyn StateView,
    ) -> Option<VMStatus> {
        None
    }
}

#[async_trait::async_trait]
impl TransactionValidation for MockVMValidator {
    type ValidationInstance = MockVMValidator;
    async fn validate_transaction(&self, txn: SignedUserTransaction) -> Result<Option<VMStatus>> {
        unimplemented!()
    }
}
