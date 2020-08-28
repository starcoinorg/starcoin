// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_executor::{peer_to_peer_txn_sent_as_association, DEFAULT_EXPIRATION_TIME};

use starcoin_consensus::Consensus;
use starcoin_vm_types::chain_config::ChainNetwork;
use types::transaction::{authenticator::AuthenticationKey, SignedUserTransaction};

pub mod chain;
pub mod chain_service;
pub mod helper;
pub mod storage;
pub mod sync;

pub fn random_txn(seq_num: u64) -> SignedUserTransaction {
    let auth_key = AuthenticationKey::random();
    peer_to_peer_txn_sent_as_association(
        auth_key.derived_address(),
        auth_key.prefix().to_vec(),
        seq_num,
        1000,
        ChainNetwork::TEST.consensus().now() + DEFAULT_EXPIRATION_TIME,
        &ChainNetwork::TEST,
    )
}
