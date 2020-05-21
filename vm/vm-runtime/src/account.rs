// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Test infrastructure for modeling Libra accounts.

use crate::genesis::GENESIS_KEYPAIR;
use crypto::ed25519::*;
use crypto::keygen::KeyGen;
use starcoin_types::{
    account_address::{self, AccountAddress},
    account_config,
    language_storage::TypeTag,
    transaction::{
        authenticator::AuthenticationKey, RawUserTransaction, Script, SignedUserTransaction,
        TransactionArgument, TransactionPayload,
    },
};
use std::time::Duration;

// TTL is 86400s. Initial time was set to 0.
pub const DEFAULT_EXPIRATION_TIME: u64 = 40_000;

/// Details about a Libra account.
///
/// Tests will typically create a set of `Account` instances to run transactions on. This type
/// encodes the logic to operate on and verify operations on any Libra account.
#[derive(Debug, Eq, PartialEq)]
pub struct Account {
    addr: AccountAddress,
    /// The current private key for this account.
    pub privkey: Ed25519PrivateKey,
    /// The current public key for this account.
    pub pubkey: Ed25519PublicKey,
}

impl Account {
    /// Creates a new account in memory.
    ///
    /// The account returned by this constructor is a purely logical entity, meaning that it does
    /// not automatically get added to the Libra store. To add an account to the store, use
    /// [`AccountData`] instances with
    /// [`FakeExecutor::add_account_data`][crate::executor::FakeExecutor::add_account_data].
    /// This function returns distinct values upon every call.
    pub fn new() -> Self {
        let (privkey, pubkey) = KeyGen::from_os_rng().generate_keypair();
        Self::with_keypair(privkey, pubkey)
    }

    /// Creates a new account with the given keypair.
    ///
    /// Like with [`Account::new`], the account returned by this constructor is a purely logical
    /// entity.
    pub fn with_keypair(privkey: Ed25519PrivateKey, pubkey: Ed25519PublicKey) -> Self {
        let addr = account_address::from_public_key(&pubkey);
        Account {
            addr,
            privkey,
            pubkey,
        }
    }

    /// Returns the address of the account. This is a hash of the public key the account was created
    /// with.
    ///
    /// The address does not change if the account's [keys are rotated][Account::rotate_key].
    pub fn address(&self) -> &AccountAddress {
        &self.addr
    }

    /// Changes the keys for this account to the provided ones.
    pub fn rotate_key(&mut self, privkey: Ed25519PrivateKey, pubkey: Ed25519PublicKey) {
        self.privkey = privkey;
        self.pubkey = pubkey;
    }

    /// Computes the authentication key for this account, as stored on the chain.
    ///
    /// This is the same as the account's address if the keys have never been rotated.
    pub fn auth_key(&self) -> Vec<u8> {
        AuthenticationKey::ed25519(&self.pubkey).to_vec()
    }

    /// Return the first 16 bytes of the account's auth key
    pub fn auth_key_prefix(&self) -> Vec<u8> {
        AuthenticationKey::ed25519(&self.pubkey).prefix().to_vec()
    }

    /// Returns a [`SignedUserTransaction`] with a payload and this account as the sender.
    ///
    /// This is the most generic way to create a transaction for testing.
    /// Max gas amount and gas unit price are ignored for WriteSet transactions.
    pub fn create_user_txn(
        &self,
        payload: TransactionPayload,
        sequence_number: u64,
        max_gas_amount: u64,
        gas_unit_price: u64,
        gas_specifier: TypeTag,
    ) -> SignedUserTransaction {
        let raw_txn = match payload {
            TransactionPayload::Module(module) => RawUserTransaction::new_module(
                *self.address(),
                sequence_number,
                module,
                max_gas_amount,
                gas_unit_price,
                gas_specifier,
                Duration::from_secs(DEFAULT_EXPIRATION_TIME),
            ),
            TransactionPayload::Script(script) => RawUserTransaction::new_script(
                *self.address(),
                sequence_number,
                script,
                max_gas_amount,
                gas_unit_price,
                gas_specifier,
                Duration::from_secs(DEFAULT_EXPIRATION_TIME),
            ),
        };

        raw_txn
            .sign(&self.privkey, self.pubkey.clone())
            .unwrap()
            .into_inner()
    }

    pub fn create_user_txn_from_raw_txn(
        &self,
        raw_txn: RawUserTransaction,
    ) -> SignedUserTransaction {
        raw_txn
            .sign(&self.privkey, self.pubkey.clone())
            .unwrap()
            .into_inner()
    }

    /// Returns a [`SignedUserTransaction`] with the arguments defined in `args` and this account as
    /// the sender.
    pub fn create_signed_txn_with_args(
        &self,
        program: Vec<u8>,
        ty_args: Vec<TypeTag>,
        args: Vec<TransactionArgument>,
        sequence_number: u64,
        max_gas_amount: u64,
        gas_unit_price: u64,
        gas_specifier: TypeTag,
    ) -> SignedUserTransaction {
        self.create_signed_txn_impl(
            *self.address(),
            TransactionPayload::Script(Script::new(program, ty_args, args)),
            sequence_number,
            max_gas_amount,
            gas_unit_price,
            gas_specifier,
        )
    }

    /// Returns a [`SignedUserTransaction`] with the arguments defined in `args` and a custom sender.
    ///
    /// The transaction is signed with the key corresponding to this account, not the custom sender.
    pub fn create_signed_txn_with_args_and_sender(
        &self,
        sender: AccountAddress,
        program: Vec<u8>,
        ty_args: Vec<TypeTag>,
        args: Vec<TransactionArgument>,
        sequence_number: u64,
        max_gas_amount: u64,
        gas_unit_price: u64,
        gas_specifier: TypeTag,
    ) -> SignedUserTransaction {
        self.create_signed_txn_impl(
            sender,
            TransactionPayload::Script(Script::new(program, ty_args, args)),
            sequence_number,
            max_gas_amount,
            gas_unit_price,
            gas_specifier,
        )
    }

    /// Returns a [`SignedUserTransaction`] with the arguments defined in `args` and a custom sender.
    ///
    /// The transaction is signed with the key corresponding to this account, not the custom sender.
    pub fn create_signed_txn_impl(
        &self,
        sender: AccountAddress,
        program: TransactionPayload,
        sequence_number: u64,
        max_gas_amount: u64,
        gas_unit_price: u64,
        gas_specifier: TypeTag,
    ) -> SignedUserTransaction {
        RawUserTransaction::new(
            sender,
            sequence_number,
            program,
            max_gas_amount,
            gas_unit_price,
            gas_specifier,
            // TTL is 86400s. Initial time was set to 0.
            Duration::from_secs(DEFAULT_EXPIRATION_TIME),
        )
        .sign(&self.privkey, self.pubkey.clone())
        .unwrap()
        .into_inner()
    }
}

impl Default for Account {
    fn default() -> Self {
        Self::new()
    }
}

pub fn create_signed_txn_with_association_account(
    program: Vec<u8>,
    ty_args: Vec<TypeTag>,
    args: Vec<TransactionArgument>,
    sequence_number: u64,
    max_gas_amount: u64,
    gas_unit_price: u64,
    gas_specifier: TypeTag,
) -> SignedUserTransaction {
    RawUserTransaction::new(
        account_config::association_address(),
        sequence_number,
        TransactionPayload::Script(Script::new(program, ty_args, args)),
        max_gas_amount,
        gas_unit_price,
        gas_specifier,
        // TTL is 86400s. Initial time was set to 0.
        Duration::from_secs(DEFAULT_EXPIRATION_TIME),
    )
    .sign(&GENESIS_KEYPAIR.0, GENESIS_KEYPAIR.1.clone())
    .unwrap()
    .into_inner()
}
