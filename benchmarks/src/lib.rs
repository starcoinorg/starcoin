// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_executor::peer_to_peer_txn_sent_as_association;
use types::transaction::{authenticator::AuthenticationKey, SignedUserTransaction};

pub mod chain;
pub mod helper;
pub mod storage;
pub mod sync;
pub mod transactions;

pub fn random_txn(seq_num: u64) -> SignedUserTransaction {
    let auth_key = AuthenticationKey::random();
    peer_to_peer_txn_sent_as_association(
        auth_key.derived_address(),
        auth_key.prefix().to_vec(),
        seq_num,
        1000,
    )
}
