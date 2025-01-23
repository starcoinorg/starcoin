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
use move_core_types::move_resource::MoveStructType;
use starcoin_crypto::ed25519::*;
use starcoin_crypto::keygen::KeyGen;
use starcoin_crypto::multi_ed25519::genesis_multi_key_pair;

use starcoin_vm_types::{
    account_config::{
        self, coin_store::CoinStoreResource, core_code_address, stc_type_tag, AccountResource,
    },
    event::EventKey,
    genesis_config::ChainId,
    identifier::Identifier,
    language_storage::ModuleId,
    language_storage::StructTag,
    state_store::state_key::StateKey,
    transaction::authenticator::{AccountPrivateKey, AccountPublicKey},
    transaction::EntryFunction,
    value::{MoveStructLayout, MoveTypeLayout},
    values::{Struct, Value},
};
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
        Self::with_keypair(privkey.into(), pubkey.into(), None)
    }

    /// Creates a new account in memory given a random seed.
    pub fn new_from_seed(seed: &mut KeyGen) -> Self {
        let (privkey, pubkey) = seed.generate_ed25519_keypair();
        Self::with_keypair(privkey.into(), pubkey.into(), None)
    }

    /// Creates a new account with the given keypair.
    ///
    /// Like with [`Account::new`], the account returned by this constructor is a purely logical
    /// entity.
    pub fn with_keypair(
        privkey: AccountPrivateKey,
        pubkey: AccountPublicKey,
        addr: Option<AccountAddress>,
    ) -> Self {
        let addr = addr.unwrap_or_else(|| pubkey.derived_address());
        Self {
            addr,
            private_key: Arc::new(privkey),
        }
    }

    /// Creates a new account in memory representing an account created in the genesis transaction.
    ///
    /// The address will be [`address`], which should be an address for a genesis account and
    /// the account will use ChainNetwork::genesis_key_pair() as its keypair.
    pub fn new_genesis_account(address: AccountAddress) -> Self {
        let (privkey, _pubkey) = genesis_key_pair();
        Self {
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
        Self {
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

    /// Returns the AccessPath that describes the Account's CoinStore resource instance.
    ///
    /// Use this to retrieve or publish the Account's CoinStore blob.
    pub fn make_coin_store_access_path(&self) -> AccessPath {
        AccessPath::resource_access_path(self.addr, CoinStoreResource::struct_tag())
    }

    /// Returns the AccessPath that describes the EventHandleGenerator resource instance.
    ///
    /// Use this to retrieve or publish the EventHandleGenerator blob.
    pub fn make_event_generator_access_path(&self) -> AccessPath {
        self.make_access_path(account_config::event_handle_generator_struct_tag())
    }

    // /// Returns the AccessPath that describes the Account balance resource instance.
    // ///
    // /// Use this to retrieve or publish the Account balance blob.
    // pub fn make_balance_access_path(&self, token_code_str: &str) -> AccessPath {
    //     let token_code =
    //         TokenCode::from_str(token_code_str).expect("token code str should been valid.");
    //     let token_type_tag = token_code
    //         .try_into()
    //         .expect("token code to type tag should be ok");
    //     // TODO/XXX: Convert this to BalanceResource::struct_tag once that takes type args
    //     self.make_access_path(BalanceResource::struct_tag_for_token(token_type_tag))
    // }

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
        // It's ok to unwrap here, because this function is only used `db-exporter` and tests
        let signature = self.private_key.sign(&raw_txn).unwrap();
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

    pub fn sign_txn(&self, raw_txn: RawUserTransaction) -> anyhow::Result<SignedUserTransaction> {
        let signature = self.private_key.sign(&raw_txn)?;
        Ok(SignedUserTransaction::new(raw_txn, signature))
    }
}

impl Default for Account {
    fn default() -> Self {
        Self::new()
    }
}

pub const DEFAULT_MAX_GAS_AMOUNT: u64 = 40000000;
pub const DEFAULT_EXPIRATION_TIME: u64 = 40_000;

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
    let args = vec![
        bcs_ext::to_bytes(receiver.address()).unwrap(),
        bcs_ext::to_bytes(&transfer_amount).unwrap(),
    ];

    // get a SignedUserTransaction
    sender.create_signed_txn_with_args(
        TransactionPayload::EntryFunction(EntryFunction::new(
            ModuleId::new(
                core_code_address(),
                Identifier::new("transfer_scripts").unwrap(),
            ),
            Identifier::new("peer_to_peer_v2").unwrap(),
            vec![stc_type_tag()],
            args,
        )),
        seq_num,
        DEFAULT_MAX_GAS_AMOUNT, // this is a default for gas
        1,                      // this is a default for gas
        expiration_timestamp_secs,
        chain_id,
    )
}

//---------------------------------------------------------------------------
// Balance resource representation
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
// CoinStore resource representation
//---------------------------------------------------------------------------

/// Struct that represents an account CoinStore resource for tests.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoinStore {
    coin: u64,
    frozen: bool,
    deposit_events: EventHandle,
    withdraw_events: EventHandle,
}

impl CoinStore {
    /// Create a new CoinStore
    pub fn new(coin: u64, deposit_events: EventHandle, withdraw_events: EventHandle) -> Self {
        Self {
            coin,
            frozen: false,
            deposit_events,
            withdraw_events,
        }
    }

    /// Retrieve the balance inside of this
    pub fn coin(&self) -> u64 {
        self.coin
    }

    /// Returns the Move Value for the account's CoinStore
    pub fn to_bytes(&self) -> Vec<u8> {
        let coin_store = CoinStoreResource::new(
            self.coin,
            self.frozen,
            self.deposit_events.clone(),
            self.withdraw_events.clone(),
        );
        bcs_ext::to_bytes(&coin_store).unwrap()
    }
}

/// Represents an account along with initial state about it.
///
/// `AccountData` captures the initial state needed to create accounts for tests.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountData {
    account: Account,
    sequence_number: u64,
    coin_register_events: EventHandle,
    key_rotation_events: EventHandle,
    coin_store: CoinStore,
}

fn new_event_handle(count: u64, address: AccountAddress) -> EventHandle {
    EventHandle::new(EventKey::new(count, address), 0)
}

impl AccountData {
    /// Creates a new `AccountData` with a new account.
    ///
    /// Most tests will want to use this constructor.
    pub fn new(balance: u128, sequence_number: u64) -> Self {
        Self::with_account(Account::new(), balance, sequence_number)
    }

    pub fn new_empty() -> Self {
        Self::with_account(Account::new(), 0, 0)
    }

    pub fn new_from_seed(seed: &mut KeyGen, balance: u128, sequence_number: u64) -> Self {
        Self::with_account(Account::new_from_seed(seed), balance, sequence_number)
    }

    pub fn with_account(account: Account, balance: u128, sequence_number: u64) -> Self {
        Self::with_account_and_event_counts(account, balance, sequence_number, 0, 0)
    }

    /// Creates a new `AccountData` with the provided account.
    // pub fn with_account(
    //     account: Account,
    //     balance: u128,
    //     balance_token_code: &str,
    //     sequence_number: u64,
    // ) -> Self {
    //     Self::with_account_and_event_counts(
    //         account,
    //         balance,
    //         balance_token_code,
    //         sequence_number,
    //         0,
    //         0,
    //         0,
    //         false,
    //         false,
    //     )
    // }

    /// Creates a new `AccountData` with the provided account.
    pub fn with_keypair(
        privkey: Ed25519PrivateKey,
        pubkey: Ed25519PublicKey,
        addr: Option<AccountAddress>,
        balance: u128,
        _balance_token_code: &str, // TODO(BobOng): [framework compatible] To support token type
        sequence_number: u64,
    ) -> Self {
        let account = Account::with_keypair(privkey.into(), pubkey.into(), addr);
        Self::with_account(account, balance, sequence_number)
    }

    /// Creates a new `AccountData` with custom parameters.
    pub fn with_account_and_event_counts(
        account: Account,
        balance: u128,
        sequence_number: u64,
        sent_events_count: u64,
        received_events_count: u64,
    ) -> Self {
        let addr = *account.address();
        Self {
            account,
            coin_store: CoinStore::new(
                balance as u64,
                new_event_handle(received_events_count, addr),
                new_event_handle(sent_events_count, addr),
            ),
            sequence_number,
            coin_register_events: new_event_handle(0, addr),
            key_rotation_events: new_event_handle(1, addr),
        }
    }
    /// Adds the balance held by this account to the one represented as balance_token_code
    // pub fn add_balance(&mut self, balance_token_code: &str) {
    //     self.coin_store
    //         .insert(balance_token_code.to_string(), Balance::new(0));
    // }

    /// Changes the keys for this account to the provided ones.
    pub fn rotate_key(&mut self, privkey: AccountPrivateKey) {
        self.account.rotate_key(privkey)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let account = AccountResource::new(
            self.sequence_number,
            self.account.auth_key().to_vec(),
            self.coin_register_events.clone(),
            self.key_rotation_events.clone(),
        );
        bcs_ext::to_bytes(&account).unwrap()
    }

    pub fn event_handle_layout() -> MoveStructLayout {
        MoveStructLayout::new(vec![
            MoveTypeLayout::U64,
            MoveTypeLayout::Vector(Box::new(MoveTypeLayout::U8)),
        ])
    }

    /// Returns the AccessPath that describes the Account resource instance.
    ///
    /// Use this to retrieve or publish the Account blob.
    pub fn make_account_access_path(&self) -> AccessPath {
        self.account.make_account_access_path()
    }

    /// Returns the AccessPath that describes the Account's CoinStore resource instance.
    ///
    /// Use this to retrieve or publish the Account's CoinStore blob.
    pub fn make_coin_store_access_path(&self) -> AccessPath {
        self.account.make_coin_store_access_path()
    }

    /// Returns the AccessPath that describes the Account balance resource instance.
    ///
    /// Use this to retrieve or publish the Account blob.
    // fn balance_resource_tag(token_code: &str) -> StructTag {
    //     let token_code =
    //         TokenCode::from_str(token_code).expect("token code str should been valid.");
    //     let token_type_tag = token_code
    //         .try_into()
    //         .expect("token code to type tag should be ok");
    //     BalanceResource::struct_tag_for_token(token_type_tag)
    // }

    //TODO create account by Move, avoid serialize data in rust.
    /// Creates a writeset that contains the account data and can be patched to the storage
    /// directly.
    pub fn to_writeset(&self) -> WriteSet {
        let write_set = vec![
            (
                StateKey::resource_typed::<AccountResource>(self.address()),
                WriteOp::legacy_modification(self.to_bytes().into()),
            ),
            (
                StateKey::resource_typed::<CoinStoreResource>(self.address()),
                WriteOp::legacy_modification(self.coin_store.to_bytes().into()),
            ),
        ];

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
    pub fn balance(&self) -> u64 {
        self.coin_store.coin()
    }

    /// Returns the initial sequence number.
    pub fn sequence_number(&self) -> u64 {
        self.sequence_number
    }

    /// Returns the initial withdraw events count.
    pub fn withdraw_events_count(&self) -> u64 {
        self.coin_store.withdraw_events.count()
    }

    /// Returns the initial deposit count.
    pub fn deposit_events_count(&self) -> u64 {
        self.coin_store.deposit_events.count()
    }

    /// Returns the initial accept token events count.
    pub fn accept_token_events_count(&self) -> u64 {
        self.coin_register_events.count()
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
