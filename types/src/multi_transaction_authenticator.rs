// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_vm_types::transaction::authenticator::TransactionAuthenticator;
use starcoin_vm2_vm_types::transaction::authenticator::TransactionAuthenticator as TransactionAuthenticatorV2;

pub enum MultiTransactionAuthenticator {
    VM1(TransactionAuthenticator),
    VM2(TransactionAuthenticatorV2),
}