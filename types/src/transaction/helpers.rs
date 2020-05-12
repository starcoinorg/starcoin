// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_address::AccountAddress,
    transaction::{RawUserTransaction, SignedUserTransaction, TransactionPayload},
};
use anyhow::Result;
use chrono::Utc;
use starcoin_crypto::{ed25519::*, hash::PlainCryptoHash, test_utils::KeyPair, traits::SigningKey};
use starcoin_vm_types::language_storage::TypeTag;

pub fn create_unsigned_txn(
    payload: TransactionPayload,
    sender_address: AccountAddress,
    sender_sequence_number: u64,
    max_gas_amount: u64,
    gas_unit_price: u64,
    gas_specifier: TypeTag,
    txn_expiration: i64, // for compatibility with UTC's timestamp.
) -> RawUserTransaction {
    RawUserTransaction::new(
        sender_address,
        sender_sequence_number,
        payload,
        max_gas_amount,
        gas_unit_price,
        gas_specifier,
        std::time::Duration::new((Utc::now().timestamp() + txn_expiration) as u64, 0),
    )
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
    gas_specifier: TypeTag,
    txn_expiration: i64, // for compatibility with UTC's timestamp.
) -> Result<SignedUserTransaction> {
    let raw_txn = create_unsigned_txn(
        payload,
        sender_address,
        sender_sequence_number,
        max_gas_amount,
        gas_unit_price,
        gas_specifier,
        txn_expiration,
    );
    signer.sign_txn(raw_txn)
}

impl TransactionSigner for KeyPair<Ed25519PrivateKey, Ed25519PublicKey> {
    fn sign_txn(&self, raw_txn: RawUserTransaction) -> Result<SignedUserTransaction> {
        let signature = self.private_key.sign_message(&raw_txn.crypto_hash());
        Ok(SignedUserTransaction::new(
            raw_txn,
            self.public_key.clone(),
            signature,
        ))
    }
}
