// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{access_path_helper::AccessPathHelper, transaction_helper::TransactionHelper};
use crypto::ed25519::*;
use libra_types::{
    account_config as libra_account_config, byte_array::ByteArray as LibraByteArray,
};
use logger::prelude::*;
use move_vm_types::{
    identifier::create_access_path,
    values::{Struct, Value},
};
use rand::{Rng, SeedableRng};
use std::time::Duration;
use types::{
    access_path::AccessPath,
    account_address::AccountAddress,
    byte_array::ByteArray,
    event::EventHandle,
    transaction::{
        RawUserTransaction, Script, SignedUserTransaction, TransactionArgument, TransactionPayload,
    },
};

#[derive(Debug, Eq, PartialEq)]
pub struct Account {
    addr: AccountAddress,
    pub privkey: Ed25519PrivateKey,
    pub pubkey: Ed25519PublicKey,
}

impl Account {
    pub fn new() -> Self {
        let mut seed_rng = rand::rngs::OsRng::new().expect("can't access OsRng");
        let seed_buf: [u8; 32] = seed_rng.gen();
        let mut rng = rand::rngs::StdRng::from_seed(seed_buf);

        let (privkey, pubkey) = compat::generate_keypair(&mut rng);
        Self::with_keypair(privkey, pubkey)
    }

    pub fn with_keypair(privkey: Ed25519PrivateKey, pubkey: Ed25519PublicKey) -> Self {
        let addr = AccountAddress::from_public_key(&pubkey);
        Account {
            addr,
            privkey,
            pubkey,
        }
    }

    pub fn address(&self) -> &AccountAddress {
        &self.addr
    }

    pub fn make_access_path(&self) -> AccessPath {
        let addr = self.address().clone();
        let access_path = create_access_path(
            &TransactionHelper::to_libra_AccountAddress(addr),
            libra_account_config::account_struct_tag(),
        );
        info!("libra access_path: {:?}", access_path);
        let ap = AccessPathHelper::to_Starcoin_AccessPath(&access_path);
        info!("starcoin access_path: {:?}", ap);
        ap
    }

    pub fn rotate_key(&mut self, privkey: Ed25519PrivateKey, pubkey: Ed25519PublicKey) {
        self.privkey = privkey;
        self.pubkey = pubkey;
    }

    pub fn auth_key(&self) -> ByteArray {
        ByteArray::new(AccountAddress::from_public_key(&self.pubkey).to_vec())
    }
}

impl Default for Account {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct AccountData {
    account: Account,
    balance: u64,
    sequence_number: u64,
    delegated_key_rotation_capability: bool,
    delegated_withdrawal_capability: bool,
    sent_events: EventHandle,
    received_events: EventHandle,
    event_generator: u64,
}

fn new_event_handle(count: u64) -> EventHandle {
    EventHandle::random_handle(count)
}

impl AccountData {
    pub fn new(balance: u64, sequence_number: u64) -> Self {
        Self::with_account(Account::new(), balance, sequence_number)
    }

    /// Creates a new `AccountData` with the provided account.
    pub fn with_account(account: Account, balance: u64, sequence_number: u64) -> Self {
        Self::with_account_and_event_counts(account, balance, sequence_number, 0, 0, false, false)
    }

    /// Creates a new `AccountData` with the provided account.
    pub fn with_keypair(
        privkey: Ed25519PrivateKey,
        pubkey: Ed25519PublicKey,
        balance: u64,
        sequence_number: u64,
    ) -> Self {
        let account = Account::with_keypair(privkey, pubkey);
        Self::with_account(account, balance, sequence_number)
    }

    /// Creates a new `AccountData` with custom parameters.
    pub fn with_account_and_event_counts(
        account: Account,
        balance: u64,
        sequence_number: u64,
        sent_events_count: u64,
        received_events_count: u64,
        delegated_key_rotation_capability: bool,
        delegated_withdrawal_capability: bool,
    ) -> Self {
        Self {
            account,
            balance,
            sequence_number,
            delegated_key_rotation_capability,
            delegated_withdrawal_capability,
            sent_events: new_event_handle(sent_events_count),
            received_events: new_event_handle(received_events_count),
            event_generator: 2,
        }
    }

    /// Changes the keys for this account to the provided ones.
    pub fn rotate_key(&mut self, privkey: Ed25519PrivateKey, pubkey: Ed25519PublicKey) {
        self.account.rotate_key(privkey, pubkey)
    }

    /// Creates and returns a resource [`Value`] for this data.
    pub fn to_resource(&self) -> Value {
        // TODO: publish some concept of Account
        let coin = Value::struct_(Struct::new(vec![Value::u64(self.balance)]));
        Value::struct_(Struct::new(vec![
            Value::byte_array(LibraByteArray::new(
                AccountAddress::from_public_key(&self.account.pubkey).to_vec(),
            )),
            coin,
            Value::bool(self.delegated_key_rotation_capability),
            Value::bool(self.delegated_withdrawal_capability),
            Value::struct_(Struct::new(vec![
                Value::u64(self.received_events.count()),
                Value::byte_array(LibraByteArray::new(self.received_events.key().to_vec())),
            ])),
            Value::struct_(Struct::new(vec![
                Value::u64(self.sent_events.count()),
                Value::byte_array(LibraByteArray::new(self.sent_events.key().to_vec())),
            ])),
            Value::u64(self.sequence_number),
            Value::struct_(Struct::new(vec![Value::u64(self.event_generator)])),
        ]))
    }

    pub fn make_access_path(&self) -> AccessPath {
        self.account.make_access_path()
    }

    pub fn address(&self) -> &AccountAddress {
        self.account.address()
    }

    /// Returns the underlying [`Account`] instance.
    pub fn account(&self) -> &Account {
        &self.account
    }

    /// Converts this data into an `Account` instance.
    pub fn into_account(self) -> Account {
        self.account
    }

    /// Returns the initial balance.
    pub fn balance(&self) -> u64 {
        self.balance
    }

    /// Returns the initial sequence number.
    pub fn sequence_number(&self) -> u64 {
        self.sequence_number
    }

    /// Returns the unique key for this sent events stream.
    pub fn sent_events_key(&self) -> &[u8] {
        self.sent_events.key().as_bytes()
    }

    /// Returns the initial sent events count.
    pub fn sent_events_count(&self) -> u64 {
        self.sent_events.count()
    }

    /// Returns the unique key for this received events stream.
    pub fn received_events_key(&self) -> &[u8] {
        self.received_events.key().as_bytes()
    }

    /// Returns the initial received events count.
    pub fn received_events_count(&self) -> u64 {
        self.received_events.count()
    }
}
