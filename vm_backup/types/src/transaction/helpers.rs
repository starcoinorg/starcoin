// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::genesis_config::ChainId;
use crate::{
    account_address::AccountAddress,
    transaction::{RawUserTransaction, SignedUserTransaction, TransactionPayload},
};
use anyhow::Result;
use starcoin_crypto::{ed25519::*, test_utils::KeyPair, traits::SigningKey};

/// Returns the number of non-leap seconds since January 1, 1970 0:00:00 UTC
/// (aka "UNIX timestamp").
pub fn get_current_timestamp() -> u64 {
    chrono::Utc::now().timestamp() as u64
}

pub trait TransactionSigner {
    fn sign_txn(&self, raw_txn: RawUserTransaction) -> Result<SignedUserTransaction>;
}

/// Craft a transaction request.
pub fn create_user_txn<T: TransactionSigner + ?Sized>(
    signer: &T,
    payload: TransactionPayload,
    sender_address: AccountAddress,
    sender_sequence_number: u64,
    max_gas_amount: u64,
    gas_unit_price: u64,
    expiration_timestamp_secs: u64,
    chain_id: ChainId,
) -> Result<SignedUserTransaction> {
    let raw_txn = RawUserTransaction::new_with_default_gas_token(
        sender_address,
        sender_sequence_number,
        payload,
        max_gas_amount,
        gas_unit_price,
        expiration_timestamp_secs,
        chain_id,
    );
    signer.sign_txn(raw_txn)
}

impl TransactionSigner for KeyPair<Ed25519PrivateKey, Ed25519PublicKey> {
    fn sign_txn(&self, raw_txn: RawUserTransaction) -> Result<SignedUserTransaction> {
        let signature = self.private_key.sign(&raw_txn);
        Ok(SignedUserTransaction::ed25519(
            raw_txn,
            self.public_key.clone(),
            signature,
        ))
    }
}
