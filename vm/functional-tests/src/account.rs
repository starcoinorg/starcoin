// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Test infrastructure for modeling Libra accounts.

use executor::{create_signed_txn_with_association_account, DEFAULT_MAX_GAS_AMOUNT};
use starcoin_config::genesis_key_pair;
use starcoin_crypto::ed25519::*;
use starcoin_crypto::keygen::KeyGen;
use starcoin_types::{
    access_path::AccessPath,
    account_address::AccountAddress,
    event::EventHandle,
    transaction::{
        authenticator::AuthenticationKey, RawUserTransaction, Script, SignedUserTransaction,
        TransactionArgument, TransactionPayload,
    },
    write_set::{WriteOp, WriteSet, WriteSetMut},
};
use starcoin_vm_runtime::starcoin_vm::DEFAULT_CURRENCY_TY;
use starcoin_vm_types::account_config::STC_TOKEN_CODE_STR;
use starcoin_vm_types::token::token_code::TokenCode;
use starcoin_vm_types::chain_config::ChainId;
use starcoin_vm_types::value::{MoveStructLayout, MoveTypeLayout};
use starcoin_vm_types::{
    account_config::stc_type_tag,
    account_config::{self, AccountResource, BalanceResource},
    language_storage::{ResourceKey, StructTag, TypeTag},
    move_resource::MoveResource,
    values::{Struct, Value},
};
use std::collections::BTreeMap;
use std::str::FromStr;
use stdlib::transaction_scripts::StdlibScript;

// TTL is 86400s. Initial time was set to 0.
pub const DEFAULT_EXPIRATION_TIME: u64 = 40_000;

/// Details about a Libra account.
///
/// Tests will typically create a set of `Account` instances to run transactions on. This type
/// encodes the logic to operate on and verify operations on any Libra account.
#[derive(Debug, Clone, Eq, PartialEq)]
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
        let addr = starcoin_types::account_address::from_public_key(&pubkey);
        Account {
            addr,
            privkey,
            pubkey,
        }
    }

    /// Creates a new account in memory representing an account created in the genesis transaction.
    ///
    /// The address will be [`address`], which should be an address for a genesis account and
    /// the account will use ChainNetwork::genesis_key_pair() as its keypair.
    pub fn new_genesis_account(address: AccountAddress) -> Self {
        let (privkey, pubkey) = genesis_key_pair();
        Account {
            addr: address,
            pubkey,
            privkey,
        }
    }

    /// Creates a new account representing the association in memory.
    ///
    /// The address will be [`association_address`][account_config::association_address], and
    /// the account will use [`GENESIS_KEYPAIR`][struct@GENESIS_KEYPAIR] as its keypair.
    pub fn new_association() -> Self {
        Self::new_genesis_account(account_config::association_address())
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
        // TODO/XXX: Convert this to BalanceResource::struct_tag once that takes type args
        self.make_access_path(BalanceResource::struct_tag_for_token_code(token_code))
    }

    // TODO: plug in the account type
    fn make_access_path(&self, tag: StructTag) -> AccessPath {
        // TODO: we need a way to get the type (FatStructType) of the Account in place
        let resource_tag = ResourceKey::new(self.addr, tag);
        AccessPath::resource_access_path(&resource_tag)
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
        expiration_timestamp_secs: u64,
        chain_id: ChainId,
    ) -> SignedUserTransaction {
        self.create_signed_txn_impl(
            *self.address(),
            TransactionPayload::Script(Script::new(program, ty_args, args)),
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
        Self::create_raw_txn_impl(
            sender,
            program,
            sequence_number,
            max_gas_amount,
            gas_unit_price,
            expiration_timestamp_secs,
            chain_id,
        )
        .sign(&self.privkey, self.pubkey.clone())
        .unwrap()
        .into_inner()
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
        RawUserTransaction::new(
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
        raw_txn
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
        Value::struct_(Struct::pack(vec![Value::u128(self.token)], true))
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
        Value::struct_(Struct::pack(
            vec![Value::u64(self.counter), Value::address(self.addr)],
            true,
        ))
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
    sent_events: EventHandle,
    received_events: EventHandle,
    balances: BTreeMap<String, Balance>,
    event_generator: EventHandleGenerator,
}

fn new_event_handle(count: u64) -> EventHandle {
    EventHandle::random_handle(count)
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
            false,
            false,
        )
    }

    /// Creates a new `AccountData` with the provided account.
    pub fn with_keypair(
        privkey: Ed25519PrivateKey,
        pubkey: Ed25519PublicKey,
        balance: u128,
        balance_token_code: &str,
        sequence_number: u64,
    ) -> Self {
        let account = Account::with_keypair(privkey, pubkey);
        Self::with_account(account, balance, balance_token_code, sequence_number)
    }

    /// Creates a new `AccountData` with custom parameters.
    pub fn with_account_and_event_counts(
        account: Account,
        balance: u128,
        balance_token_code: &str,
        sequence_number: u64,
        sent_events_count: u64,
        received_events_count: u64,
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
        Self {
            event_generator: EventHandleGenerator::new_with_event_count(*account.address(), 2),
            account,
            balances,
            sequence_number,
            key_rotation_capability,
            withdrawal_capability,
            sent_events: new_event_handle(sent_events_count),
            received_events: new_event_handle(received_events_count),
        }
    }

    /// Adds the balance held by this account to the one represented as balance_token_code
    pub fn add_balance(&mut self, balance_token_code: &str) {
        self.balances
            .insert(balance_token_code.to_string(), Balance::new(0));
    }

    /// Changes the keys for this account to the provided ones.
    pub fn rotate_key(&mut self, privkey: Ed25519PrivateKey, pubkey: Ed25519PublicKey) {
        self.account.rotate_key(privkey, pubkey)
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
        let account = Value::struct_(Struct::pack(
            vec![
                // TODO: this needs to compute the auth key instead
                Value::vector_u8(AuthenticationKey::ed25519(&self.account.pubkey).to_vec()),
                self.withdrawal_capability.as_ref().unwrap().value(),
                self.key_rotation_capability.as_ref().unwrap().value(),
                Value::struct_(Struct::pack(
                    vec![
                        Value::u64(self.received_events.count()),
                        Value::vector_u8(self.received_events.key().to_vec()),
                    ],
                    true,
                )),
                Value::struct_(Struct::pack(
                    vec![
                        Value::u64(self.sent_events.count()),
                        Value::vector_u8(self.sent_events.key().to_vec()),
                    ],
                    true,
                )),
                Value::u64(self.sequence_number),
            ],
            true,
        ));
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
        Value::vector_resource_for_testing_only(vec![Value::struct_(Struct::pack(
            vec![Value::address(self.account_address)],
            true,
        ))])
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
        Value::vector_resource_for_testing_only(vec![Value::struct_(Struct::pack(
            vec![Value::address(self.account_address)],
            true,
        ))])
    }
}

/// Returns a transaction to transfer coin from one account to another (possibly new) one, with the
/// given arguments.
pub fn peer_to_peer_txn(
    sender: &Account,
    receiver: &Account,
    seq_num: u64,
    transfer_amount: u128,
    expiration_timestamp_secs: u64,
    chain_id: ChainId,
) -> SignedUserTransaction {
    let mut args: Vec<TransactionArgument> = Vec::new();
    args.push(TransactionArgument::Address(*receiver.address()));
    args.push(TransactionArgument::U8Vector(receiver.auth_key_prefix()));
    args.push(TransactionArgument::U128(transfer_amount));

    // get a SignedTransaction
    sender.create_signed_txn_with_args(
        StdlibScript::PeerToPeer.compiled_bytes().into_vec(),
        vec![stc_type_tag()],
        args,
        seq_num,
        DEFAULT_MAX_GAS_AMOUNT, // this is a default for gas
        1,                      // this is a default for gas
        expiration_timestamp_secs,
        chain_id,
    )
}

/// Returns a transaction to create a new account with the given arguments.
pub fn create_account_txn_sent_as_association(
    new_account: &Account,
    seq_num: u64,
    initial_amount: u128,
    expiration_timstamp_secs: u64,
    chain_id: ChainId,
) -> SignedUserTransaction {
    let mut args: Vec<TransactionArgument> = Vec::new();
    args.push(TransactionArgument::Address(*new_account.address()));
    args.push(TransactionArgument::U8Vector(new_account.auth_key_prefix()));
    args.push(TransactionArgument::U128(initial_amount));

    create_signed_txn_with_association_account(
        TransactionPayload::Script(Script::new(
            StdlibScript::CreateAccount.compiled_bytes().into_vec(),
            vec![DEFAULT_CURRENCY_TY.clone()],
            args,
        )),
        seq_num,
        DEFAULT_MAX_GAS_AMOUNT,
        1,
        expiration_timstamp_secs,
        chain_id,
    )
}
