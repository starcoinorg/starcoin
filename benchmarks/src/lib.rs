// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_executor::{peer_to_peer_txn_sent_as_association, DEFAULT_EXPIRATION_TIME};

use crypto::ed25519::random_public_key;
use starcoin_config::ChainNetwork;
use starcoin_vm_types::account_address;
use types::transaction::SignedUserTransaction;

pub mod chain;
pub mod helper;
pub mod storage;

pub fn random_txn(seq_num: u64, net: &ChainNetwork) -> SignedUserTransaction {
    let random_public_key = random_public_key();
    let addr = account_address::from_public_key(&random_public_key);
    peer_to_peer_txn_sent_as_association(
        addr,
        seq_num,
        1000,
        net.time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
        net,
    )
}
