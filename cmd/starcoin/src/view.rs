// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use starcoin_crypto::{
    ed25519::{Ed25519PublicKey, Ed25519Signature},
    hash::CryptoHash,
    HashValue,
};
use starcoin_types::{account_address::AccountAddress, transaction::SignedUserTransaction};
use starcoin_wallet_api::WalletAccount;

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountWithStateView {
    pub account: WalletAccount,
    pub sequence_number: Option<u64>,
    pub balance: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionView {
    pub id: HashValue,
    pub sender: AccountAddress,
    pub sequence_number: u64,
    pub gas_unit_price: u64,
    pub max_gas_amount: u64,
    pub public_key: Ed25519PublicKey,
    pub signature: Ed25519Signature,
}

impl From<SignedUserTransaction> for TransactionView {
    fn from(txn: SignedUserTransaction) -> Self {
        Self {
            id: txn.raw_txn().crypto_hash(),
            sender: txn.sender(),
            sequence_number: txn.sequence_number(),
            gas_unit_price: txn.gas_unit_price(),
            max_gas_amount: txn.max_gas_amount(),
            public_key: txn.public_key(),
            signature: txn.signature(),
        }
    }
}
