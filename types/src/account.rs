// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Test infrastructure for modeling Diem accounts.

use crate::{
    access_path::AccessPath,
    account_address::AccountAddress,
    event::EventHandle,
    transaction::{
        authenticator::AuthenticationKey, RawUserTransaction, SignedUserTransaction,
        TransactionPayload,
    },
    write_set::{WriteOp, WriteSet, WriteSetMut},
};
use starcoin_crypto::ed25519::*;
use starcoin_crypto::keygen::KeyGen;
use starcoin_crypto::multi_ed25519::genesis_multi_key_pair;
use starcoin_vm_types::account_config::STC_TOKEN_CODE_STR;
use starcoin_vm_types::genesis_config::ChainId;
use starcoin_vm_types::token::token_code::TokenCode;
use starcoin_vm_types::value::{MoveStructLayout, MoveTypeLayout};
use starcoin_vm_types::{
    account_config::{self, AccountResource, BalanceResource},
    language_storage::StructTag,
    move_resource::MoveResource,
    transaction::authenticator::{AccountPrivateKey, AccountPublicKey},
    values::{Struct, Value},
};
use std::collections::BTreeMap;
use std::convert::TryInto;
use std::str::FromStr;
use std::sync::Arc;

/// Details about a Starcoin account.
///
/// Tests will typically create a set of `Account` instances to run transactions on. This type
/// encodes the logic to operate on and verify operations on any Diem account.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Account {
    pub addr: AccountAddress,
    private_key: Arc<AccountPrivateKey>,
}

impl Account {
    /// Creates a new account in memory.
    ///
    /// The account returned by this constructor is a purely logical entity, meaning that it does
    /// not automatically get added to the Diem store. To add an account to the store, use
    /// [`AccountData`] instances with
    /// [`FakeExecutor::add_account_data`][crate::executor::FakeExecutor::add_account_data].
    /// This function returns distinct values upon every call.
    pub fn new() -> Self {
        let (privkey, pubkey) = KeyGen::from_os_rng().generate_keypair();
        Self::with_keypair(privkey, pubkey, None)
    }

    /// Creates a new account with the given keypair.
    ///
    /// Like with [`Account::new`], the account returned by this constructor is a purely logical
    /// entity.
    pub fn with_keypair(
        privkey: Ed25519PrivateKey,
        pubkey: Ed25519PublicKey,
        addr: Option<AccountAddress>,
    ) -> Self {
        let addr = addr.unwrap_or_else(|| crate::account_address::from_public_key(&pubkey));
        Account {
            addr,
            private_key: Arc::new(AccountPrivateKey::Single(privkey)),
        }
    }

    /// Creates a new account in memory representing an account created in the genesis transaction.
    ///
    /// The address will be [`address`], which should be an address for a genesis account and
    /// the account will use ChainNetwork::genesis_key_pair() as its keypair.
    pub fn new_genesis_account(address: AccountAddress) -> Self {
        let (privkey, _pubkey) = genesis_key_pair();
        Account {
            addr: address,
            private_key: Arc::new(AccountPrivateKey::Single(privkey)),
        }
    }

    /// Creates a new account representing the association in memory.
    ///
    /// The address will be [`association_address`][account_config::association_address], and
    /// the account will use [`GENESIS_KEYPAIR`][struct@GENESIS_KEYPAIR] as its keypair.
    pub fn new_association() -> Self {
        let (privkey, _pubkey) = genesis_multi_key_pair();
        Account {
            addr: account_config::association_address(),
            private_key: Arc::new(AccountPrivateKey::Multi(privkey)),
        }
    }

    pub fn private_key(&self) -> &AccountPrivateKey {
        &self.private_key
    }

    pub fn auth_key(&self) -> AuthenticationKey {
        self.private_key.public_key().authentication_key()
    }

    pub fn public_key(&self) -> AccountPublicKey {
        self.private_key.public_key()
    }

    /// Returns the address of the account. This is a hash of the public key the account was created
    /// with.
    ///
    /// The address does not change if the account's [keys are rotated][Account::rotate_key].
    pub fn address(&self) -> &AccountAddress {
        &self.addr
    }

    /// Returns the AccessPath that describes the Account resource instance.
    ///
    /// Use this to retrieve or publish the Account blob.
    pub fn make_account_access_path(&self) -> AccessPath {
        self.make_access_path(AccountResource::struct_tag())
    }

    /// Returns the AccessPath that describes the EventHandleGenerator resource instance.
    ///
    /// Use this to retrieve or publish the EventHandleGenerator blob.
    pub fn make_event_generator_access_path(&self) -> AccessPath {
        self.make_access_path(account_config::event_handle_generator_struct_tag())
    }

    /// Returns the AccessPath that describes the Account balance resource instance.
    ///
    /// Use this to retrieve or publish the Account balance blob.
    pub fn make_balance_access_path(&self, token_code_str: &str) -> AccessPath {
        let token_code =
            TokenCode::from_str(token_code_str).expect("token code str should been valid.");
        let token_type_tag = token_code
            .try_into()
            .expect("token code to type tag should be ok");
        // TODO/XXX: Convert this to BalanceResource::struct_tag once that takes type args
        self.make_access_path(BalanceResource::struct_tag_for_token(token_type_tag))
    }

    fn make_access_path(&self, tag: StructTag) -> AccessPath {
        // TODO: we need a way to get the type (FatStructType) of the Account in place
        AccessPath::resource_access_path(self.addr, tag)
    }

    /// Changes the keys for this account to the provided ones.
    pub fn rotate_key(&mut self, privkey: AccountPrivateKey) {
        self.private_key = Arc::new(privkey);
    }

    /// Returns a [`SignedUserTransaction`] with the arguments defined in `args` and this account as
    /// the sender.
    pub fn create_signed_txn_with_args(
        &self,
        program: TransactionPayload,
        sequence_number: u64,
        max_gas_amount: u64,
        gas_unit_price: u64,
        expiration_timestamp_secs: u64,
        chain_id: ChainId,
    ) -> SignedUserTransaction {
        self.create_signed_txn_impl(
            *self.address(),
            program,
            sequence_number,
            max_gas_amount,
            gas_unit_price,
            expiration_timestamp_secs,
            chain_id,
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
        expiration_timestamp_secs: u64,
        chain_id: ChainId,
    ) -> SignedUserTransaction {
        let raw_txn = Self::create_raw_txn_impl(
            sender,
            program,
            sequence_number,
            max_gas_amount,
            gas_unit_price,
            expiration_timestamp_secs,
            chain_id,
        );
        let signature = self.private_key.sign(&raw_txn);
        SignedUserTransaction::new(raw_txn, signature)
    }

    // get_current_timestamp() + DEFAULT_EXPIRATION_TIME,
    pub fn create_raw_txn_impl(
        sender: AccountAddress,
        program: TransactionPayload,
        sequence_number: u64,
        max_gas_amount: u64,
        gas_unit_price: u64,
        expiration_timestamp_secs: u64,
        chain_id: ChainId,
    ) -> RawUserTransaction {
        RawUserTransaction::new_with_default_gas_token(
            sender,
            sequence_number,
            program,
            max_gas_amount,
            gas_unit_price,
            expiration_timestamp_secs,
            chain_id,
        )
    }

    pub fn sign_txn(&self, raw_txn: RawUserTransaction) -> SignedUserTransaction {
        let signature = self.private_key.sign(&raw_txn);
        SignedUserTransaction::new(raw_txn, signature)
    }
}

impl Default for Account {
    fn default() -> Self {
        Self::new()
    }
}

//---------------------------------------------------------------------------
// Balance resource represenation
//---------------------------------------------------------------------------

/// Struct that represents an account balance resource for tests.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Balance {
    token: u128,
}

impl Balance {
    /// Create a new balance with amount `balance`
    pub fn new(token: u128) -> Self {
        Self { token }
    }

    /// Retrieve the balance inside of this
    pub fn token(&self) -> u128 {
        self.token
    }

    /// Returns the Move Value for the account balance
    pub fn to_value(&self) -> Value {
        Value::struct_(Struct::pack(vec![Value::u128(self.token)]))
    }

    /// Returns the value layout for the account balance
    pub fn layout() -> MoveStructLayout {
        MoveStructLayout::new(vec![MoveTypeLayout::U128])
    }
}

//---------------------------------------------------------------------------
// Event generator resource represenation
//---------------------------------------------------------------------------

/// Struct that represents the event generator resource stored under accounts

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventHandleGenerator {
    counter: u64,
    addr: AccountAddress,
}

impl EventHandleGenerator {
    pub fn new(addr: AccountAddress) -> Self {
        Self { addr, counter: 0 }
    }

    pub fn new_with_event_count(addr: AccountAddress, counter: u64) -> Self {
        Self { addr, counter }
    }

    pub fn to_value(&self) -> Value {
        Value::struct_(Struct::pack(vec![
            Value::u64(self.counter),
            Value::address(self.addr),
        ]))
    }
    pub fn layout() -> MoveStructLayout {
        MoveStructLayout::new(vec![MoveTypeLayout::U64, MoveTypeLayout::Address])
    }
}

/// Represents an account along with initial state about it.
///
/// `AccountData` captures the initial state needed to create accounts for tests.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountData {
    account: Account,
    sequence_number: u64,
    key_rotation_capability: Option<KeyRotationCapability>,
    withdrawal_capability: Option<WithdrawCapability>,
    withdraw_events: EventHandle,
    deposit_events: EventHandle,
    accept_token_events: EventHandle,
    balances: BTreeMap<String, Balance>,
    event_generator: EventHandleGenerator,
}

impl AccountData {
    /// Creates a new `AccountData` with a new account.
    ///
    /// Most tests will want to use this constructor.
    pub fn new(balance: u128, sequence_number: u64) -> Self {
        Self::with_account(Account::new(), balance, STC_TOKEN_CODE_STR, sequence_number)
    }

    pub fn new_empty() -> Self {
        Self::with_account(Account::new(), 0, STC_TOKEN_CODE_STR, 0)
    }

    /// Creates a new `AccountData` with the provided account.
    pub fn with_account(
        account: Account,
        balance: u128,
        balance_token_code: &str,
        sequence_number: u64,
    ) -> Self {
        Self::with_account_and_event_counts(
            account,
            balance,
            balance_token_code,
            sequence_number,
            0,
            0,
            0,
            false,
            false,
        )
    }

    /// Creates a new `AccountData` with the provided account.
    pub fn with_keypair(
        privkey: Ed25519PrivateKey,
        pubkey: Ed25519PublicKey,
        addr: Option<AccountAddress>,
        balance: u128,
        balance_token_code: &str,
        sequence_number: u64,
    ) -> Self {
        let account = Account::with_keypair(privkey, pubkey, addr);
        Self::with_account(account, balance, balance_token_code, sequence_number)
    }

    /// Creates a new `AccountData` with custom parameters.
    pub fn with_account_and_event_counts(
        account: Account,
        balance: u128,
        balance_token_code: &str,
        sequence_number: u64,
        withdraw_events_count: u64,
        deposit_events_count: u64,
        accept_token_events_count: u64,
        delegated_key_rotation_capability: bool,
        delegated_withdrawal_capability: bool,
    ) -> Self {
        let mut balances = BTreeMap::new();
        balances.insert(balance_token_code.to_string(), Balance::new(balance));

        let key_rotation_capability = if delegated_key_rotation_capability {
            None
        } else {
            Some(KeyRotationCapability::new(account.addr))
        };
        let withdrawal_capability = if delegated_withdrawal_capability {
            None
        } else {
            Some(WithdrawCapability::new(account.addr))
        };
        let account_address = *account.address();
        Self {
            event_generator: EventHandleGenerator::new_with_event_count(account_address, 3),
            account,
            balances,
            sequence_number,
            key_rotation_capability,
            withdrawal_capability,
            withdraw_events: EventHandle::new_from_address(&account_address, withdraw_events_count),
            deposit_events: EventHandle::new_from_address(&account_address, deposit_events_count),
            accept_token_events: EventHandle::new_from_address(
                &account_address,
                accept_token_events_count,
            ),
        }
    }

    /// Adds the balance held by this account to the one represented as balance_token_code
    pub fn add_balance(&mut self, balance_token_code: &str) {
        self.balances
            .insert(balance_token_code.to_string(), Balance::new(0));
    }

    /// Changes the keys for this account to the provided ones.
    pub fn rotate_key(&mut self, privkey: AccountPrivateKey) {
        self.account.rotate_key(privkey)
    }

    pub fn sent_payment_event_layout() -> MoveStructLayout {
        MoveStructLayout::new(vec![
            MoveTypeLayout::U128,
            MoveTypeLayout::Address,
            MoveTypeLayout::Vector(Box::new(MoveTypeLayout::U8)),
        ])
    }

    pub fn received_payment_event_type() -> MoveStructLayout {
        MoveStructLayout::new(vec![
            MoveTypeLayout::U128,
            MoveTypeLayout::Address,
            MoveTypeLayout::Vector(Box::new(MoveTypeLayout::U8)),
        ])
    }

    pub fn event_handle_layout() -> MoveStructLayout {
        MoveStructLayout::new(vec![
            MoveTypeLayout::U64,
            MoveTypeLayout::Vector(Box::new(MoveTypeLayout::U8)),
        ])
    }

    /// Returns the (Move value) layout of the Account::Account struct
    pub fn layout() -> MoveStructLayout {
        use MoveStructLayout as S;
        use MoveTypeLayout as T;

        S::new(vec![
            T::Vector(Box::new(T::U8)),
            T::Vector(Box::new(T::Struct(WithdrawCapability::layout()))),
            T::Vector(Box::new(T::Struct(KeyRotationCapability::layout()))),
            T::Struct(Self::event_handle_layout()),
            T::Struct(Self::event_handle_layout()),
            T::Struct(Self::event_handle_layout()),
            T::U64,
        ])
    }

    /// Creates and returns the top-level resources to be published under the account
    pub fn to_value(&self) -> (Value, Vec<(String, Value)>, Value) {
        // TODO: publish some concept of Account
        let balances: Vec<_> = self
            .balances
            .iter()
            .map(|(code, balance)| (code.clone(), balance.to_value()))
            .collect();
        let event_generator = self.event_generator.to_value();
        let account = Value::struct_(Struct::pack(vec![
            // TODO: this needs to compute the auth key instead
            Value::vector_u8(self.account.auth_key().to_vec()),
            self.withdrawal_capability
                .as_ref()
                .map(|v| v.value())
                .unwrap_or_else(|| Value::vector_for_testing_only(vec![])),
            self.key_rotation_capability
                .as_ref()
                .map(|v| v.value())
                .unwrap_or_else(|| Value::vector_for_testing_only(vec![])),
            Value::struct_(Struct::pack(vec![
                Value::u64(self.withdraw_events.count()),
                Value::vector_u8(self.withdraw_events.key().to_vec()),
            ])),
            Value::struct_(Struct::pack(vec![
                Value::u64(self.deposit_events.count()),
                Value::vector_u8(self.deposit_events.key().to_vec()),
            ])),
            Value::struct_(Struct::pack(vec![
                Value::u64(self.accept_token_events.count()),
                Value::vector_u8(self.accept_token_events.key().to_vec()),
            ])),
            Value::u64(self.sequence_number),
        ]));
        (account, balances, event_generator)
    }

    /// Returns the AccessPath that describes the Account resource instance.
    ///
    /// Use this to retrieve or publish the Account blob.
    pub fn make_account_access_path(&self) -> AccessPath {
        self.account.make_account_access_path()
    }

    /// Returns the AccessPath that describes the Account balance resource instance.
    ///
    /// Use this to retrieve or publish the Account blob.
    pub fn make_balance_access_path(&self, token_code: &str) -> AccessPath {
        self.account.make_balance_access_path(token_code)
    }

    /// Returns the AccessPath that describes the EventHandleGenerator resource instance.
    ///
    /// Use this to retrieve or publish the EventHandleGenerator blob.
    pub fn make_event_generator_access_path(&self) -> AccessPath {
        self.account.make_event_generator_access_path()
    }

    //TODO create account by Move, avoid serialize data in rust.
    /// Creates a writeset that contains the account data and can be patched to the storage
    /// directly.
    pub fn to_writeset(&self) -> WriteSet {
        let (account_blob, balance_blobs, event_generator_blob) = self.to_value();
        let mut write_set = Vec::new();
        let account = account_blob
            .value_as::<Struct>()
            .unwrap()
            .simple_serialize(&AccountData::layout())
            .unwrap();
        write_set.push((self.make_account_access_path(), WriteOp::Value(account)));
        for (code, balance_blob) in balance_blobs.into_iter() {
            let balance = balance_blob
                .value_as::<Struct>()
                .unwrap()
                .simple_serialize(&Balance::layout())
                .unwrap();
            write_set.push((
                self.make_balance_access_path(code.as_str()),
                WriteOp::Value(balance),
            ));
        }

        let event_generator = event_generator_blob
            .value_as::<Struct>()
            .unwrap()
            .simple_serialize(&EventHandleGenerator::layout())
            .unwrap();
        write_set.push((
            self.make_event_generator_access_path(),
            WriteOp::Value(event_generator),
        ));
        WriteSetMut::new(write_set).freeze().unwrap()
    }

    /// Returns the address of the account. This is a hash of the public key the account was created
    /// with.
    ///
    /// The address does not change if the account's [keys are rotated][AccountData::rotate_key].
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
    pub fn balance(&self, token_code: &str) -> u128 {
        self.balances
            .get(token_code)
            .expect("get balance by currency_code fail")
            .token()
    }

    /// Returns the initial sequence number.
    pub fn sequence_number(&self) -> u64 {
        self.sequence_number
    }

    /// Returns the unique key for this withdraw events stream.
    pub fn withdraw_events_key(&self) -> &[u8] {
        self.withdraw_events.key().as_bytes()
    }

    /// Returns the initial withdraw events count.
    pub fn withdraw_events_count(&self) -> u64 {
        self.withdraw_events.count()
    }

    /// Returns the unique key for this deposit events stream.
    pub fn deposit_events_key(&self) -> &[u8] {
        self.deposit_events.key().as_bytes()
    }

    /// Returns the initial deposit count.
    pub fn deposit_events_count(&self) -> u64 {
        self.deposit_events.count()
    }

    /// Returns the initial accept token events count.
    pub fn accept_token_events_count(&self) -> u64 {
        self.accept_token_events.count()
    }

    /// Returns the unique key for this accept_token events stream.
    pub fn accept_token_events_key(&self) -> &[u8] {
        self.accept_token_events.key().as_bytes()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WithdrawCapability {
    account_address: AccountAddress,
}
impl WithdrawCapability {
    pub fn new(account_address: AccountAddress) -> Self {
        Self { account_address }
    }

    pub fn layout() -> MoveStructLayout {
        MoveStructLayout::new(vec![MoveTypeLayout::Address])
    }

    pub fn value(&self) -> Value {
        Value::vector_for_testing_only(vec![Value::struct_(Struct::pack(vec![Value::address(
            self.account_address,
        )]))])
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KeyRotationCapability {
    account_address: AccountAddress,
}
impl KeyRotationCapability {
    pub fn new(account_address: AccountAddress) -> Self {
        Self { account_address }
    }

    pub fn layout() -> MoveStructLayout {
        MoveStructLayout::new(vec![MoveTypeLayout::Address])
    }

    pub fn value(&self) -> Value {
        Value::vector_for_testing_only(vec![Value::struct_(Struct::pack(vec![Value::address(
            self.account_address,
        )]))])
    }
}
