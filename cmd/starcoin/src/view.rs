// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use forkable_jellyfish_merkle::proof::SparseMerkleProof;
use serde::{Deserialize, Serialize};
use starcoin_crypto::{hash::PlainCryptoHash, HashValue};
use starcoin_state_api::StateWithProof;
use starcoin_types::block::Block;
use starcoin_types::{account_address::AccountAddress, transaction::SignedUserTransaction};
use starcoin_wallet_api::WalletAccount;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountWithStateView {
    pub account: WalletAccount,
    // hex encoded bytes
    pub auth_key_prefix: String,
    pub sequence_number: Option<u64>,
    pub balances: HashMap<String, u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountView {
    pub sequence_number: Option<u64>,
    pub balance: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StateWithProofView {
    pub state: String,
    pub account_proof: SparseMerkleProof,
    pub account_state_proof: SparseMerkleProof,
}
impl From<StateWithProof> for StateWithProofView {
    fn from(state_proof: StateWithProof) -> Self {
        let account_state = hex::encode(state_proof.state.unwrap());
        Self {
            state: account_state,
            account_proof: state_proof.proof.account_proof,
            account_state_proof: state_proof.proof.account_state_proof,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockView {
    pub parent_hash: HashValue,
    pub number: u64,
    pub author: AccountAddress,
    pub accumulator_root: HashValue,
    pub state_root: HashValue,
    pub gas_used: u64,
}

impl From<Block> for BlockView {
    fn from(block: Block) -> Self {
        Self {
            parent_hash: block.header().parent_hash(),
            number: block.header().number(),
            author: block.header().author(),
            accumulator_root: block.header().accumulator_root(),
            state_root: block.header().state_root(),
            gas_used: block.header().gas_used(),
        }
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionView {
    pub id: HashValue,
    pub sender: AccountAddress,
    pub sequence_number: u64,
    pub gas_unit_price: u64,
    pub max_gas_amount: u64,
}

impl From<SignedUserTransaction> for TransactionView {
    fn from(txn: SignedUserTransaction) -> Self {
        Self {
            id: txn.raw_txn().crypto_hash(),
            sender: txn.sender(),
            sequence_number: txn.sequence_number(),
            gas_unit_price: txn.gas_unit_price(),
            max_gas_amount: txn.max_gas_amount(),
        }
    }
}
