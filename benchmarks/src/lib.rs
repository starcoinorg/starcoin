// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_vm_runtime::{
    account::Account, common_transactions::peer_to_peer_txn_sent_as_association,
};
use types::transaction::SignedUserTransaction;

pub mod chain;
pub mod helper;
pub mod storage;
pub mod sync;
pub mod transactions;

pub fn random_txn(seq_num: u64) -> SignedUserTransaction {
    let account = Account::new();
    peer_to_peer_txn_sent_as_association(
        account.address().clone(),
        account.auth_key_prefix(),
        seq_num,
        1000,
    )
}
