// Copyright (c) Starcoin
// SPDX-License-Identifier: Apache-2.0

//! Test infrastructure for modeling Starcoin accounts.

use crate::gas_costs;
use anyhow::{Error, Result};
use starcoin_crypto::ed25519::{Ed25519PrivateKey, Ed25519PublicKey};
use starcoin_crypto::keygen::KeyGen;
use starcoin_vm_types::access_path::AccessPath;
use starcoin_vm_types::account_address::AccountAddress;
use starcoin_vm_types::account_config::{genesis_address, AccountResource, BalanceResource};
use starcoin_vm_types::event::EventHandle;
use starcoin_vm_types::genesis_config::ChainId;
use starcoin_vm_types::language_storage::StructTag;
use starcoin_vm_types::move_resource::MoveResource;
use starcoin_vm_types::state_store::state_key::StateKey;
use starcoin_vm_types::transaction::authenticator::AuthenticationKey;
use starcoin_vm_types::transaction::{
    RawUserTransaction, Script, ScriptFunction, SignedUserTransaction, TransactionPayload,
};
use starcoin_vm_types::value::{MoveStructLayout, MoveTypeLayout};
use starcoin_vm_types::values::{Struct, Value};
use starcoin_vm_types::write_set::{WriteOp, WriteSet, WriteSetMut};
use std::str::FromStr;

// TTL is 86400s. Initial time was set to 0.
pub const DEFAULT_EXPIRATION_TIME: u64 = 40_000;

/// Details about a Aptos account.
///
/// Tests will typically create a set of `Account` instances to run transactions on. This type
/// encodes the logic to operate on and verify operations on any Aptos account.
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
    /// not automatically get added to the Aptos store. To add an account to the store, use
    /// [`AccountData`] instances with
    /// [`FakeExecutor::add_account_data`][crate::executor::FakeExecutor::add_account_data].
    /// This function returns distinct values upon every call.
    pub fn new() -> Self {
        let (privkey, pubkey) = KeyGen::from_os_rng().generate_keypair();
        Self::with_keypair(privkey, pubkey)
    }

    /// Creates a new account in memory given a random seed.
    pub fn new_from_seed(seed: &mut KeyGen) -> Self {
        let (privkey, pubkey) = seed.generate_keypair();
        Self::with_keypair(privkey, pubkey)
    }

    /// Creates a new account with the given keypair.
    ///
    /// Like with [`Account::new`], the account returned by this constructor is a purely logical
    /// entity.
    pub fn with_keypair(privkey: Ed25519PrivateKey, pubkey: Ed25519PublicKey) -> Self {
        let addr = starcoin_vm_types::account_address::from_public_key(&pubkey);
        Account {
            addr,
            privkey,
            pubkey,
        }
    }

    /// Creates a new account with the given addr and key pair
    ///
    /// Like with [`Account::new`], the account returned by this constructor is a purely logical
    /// entity.
    pub fn new_validator(
        addr: AccountAddress,
        privkey: Ed25519PrivateKey,
        pubkey: Ed25519PublicKey,
    ) -> Self {
        Account {
            addr,
            privkey,
            pubkey,
        }
    }

    /// Creates a new account in memory representing an account created in the genesis transaction.
    ///
    /// The address will be [`address`], which should be an address for a genesis account and
    /// the account will use [`GENESIS_KEYPAIR`][struct@GENESIS_KEYPAIR] as its keypair.
    pub fn new_genesis_account(address: AccountAddress) -> Self {
        // Account {
        //     addr: address,
        //     pubkey: GENESIS_KEYPAIR.1.clone(),
        //     privkey: GENESIS_KEYPAIR.0.clone(),
        // }
        let (privkey, pubkey) = KeyGen::from_os_rng().generate_keypair();
        Account {
            addr: address,
            pubkey,
            privkey,
        }
    }

    /// Creates a new account representing the aptos root account in memory.
    ///
    /// The address will be [`starcoin_root_address`][account_config::starcoin_root_address], and
    /// the account will use [`GENESIS_KEYPAIR`][struct@GENESIS_KEYPAIR] as its keypair.
    pub fn new_starcoin_root() -> Self {
        Self::new_genesis_account(genesis_address())
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
    /// Use this to retrieve or publish the Account CoinStore blob.
    pub fn make_coin_store_access_path(&self) -> AccessPath {
        // TODO(BobOng):
        // self.make_access_path(CoinStoreResource::struct_tag())
        self.make_access_path(BalanceResource::struct_tag())
    }

    // TODO: plug in the account type
    pub fn make_access_path(&self, tag: StructTag) -> AccessPath {
        // TODO: we need a way to get the type (FatStructType) of the Account in place
        // let resource_tag = ResourceKey::new(self.addr, tag);
        AccessPath::resource_access_path(self.addr, tag)
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

    pub fn transaction(&self) -> TransactionBuilder {
        TransactionBuilder::new(self.clone())
    }
}

impl Default for Account {
    fn default() -> Self {
        Self::new()
    }
}

pub struct TransactionBuilder {
    pub sender: Account,
    pub secondary_signers: Vec<Account>,
    pub sequence_number: Option<u64>,
    pub program: Option<TransactionPayload>,
    pub max_gas_amount: Option<u64>,
    pub gas_unit_price: Option<u64>,
    pub chain_id: Option<ChainId>,
    pub ttl: Option<u64>,
}

impl TransactionBuilder {
    pub fn new(sender: Account) -> Self {
        Self {
            sender,
            secondary_signers: Vec::new(),
            sequence_number: None,
            program: None,
            max_gas_amount: None,
            gas_unit_price: None,
            chain_id: None,
            ttl: None,
        }
    }

    pub fn secondary_signers(mut self, secondary_signers: Vec<Account>) -> Self {
        self.secondary_signers = secondary_signers;
        self
    }

    pub fn sequence_number(mut self, sequence_number: u64) -> Self {
        self.sequence_number = Some(sequence_number);
        self
    }

    pub fn chain_id(mut self, id: ChainId) -> Self {
        self.chain_id = Some(id);
        self
    }

    pub fn payload(mut self, payload: TransactionPayload) -> Self {
        self.program = Some(payload);
        self
    }

    pub fn script(mut self, s: Script) -> Self {
        self.program = Some(TransactionPayload::Script(s));
        self
    }

    pub fn script_function(mut self, f: ScriptFunction) -> Self {
        self.program = Some(TransactionPayload::ScriptFunction(f));
        self
    }

    // pub fn module(mut self, m: Module) -> Self {
    //     self.program = Some(TransactionPayload::ModuleBundle(ModuleBundle::from(m)));
    //     self
    // }

    // TODO(BobOng): e2e-test
    // pub fn write_set(mut self, w: WriteSetPayload) -> Self {
    //     self.program = Some(TransactionPayload::WriteSet(w));
    //     self
    // }

    pub fn max_gas_amount(mut self, max_gas_amount: u64) -> Self {
        self.max_gas_amount = Some(max_gas_amount);
        self
    }

    pub fn gas_unit_price(mut self, gas_unit_price: u64) -> Self {
        self.gas_unit_price = Some(gas_unit_price);
        self
    }

    pub fn ttl(mut self, ttl: u64) -> Self {
        self.ttl = Some(ttl);
        self
    }

    pub fn raw(self) -> RawUserTransaction {
        RawUserTransaction::new_with_default_gas_token(
            *self.sender.address(),
            self.sequence_number.expect("sequence number not set"),
            self.program.expect("transaction payload not set"),
            self.max_gas_amount.unwrap_or(gas_costs::TXN_RESERVED),
            self.gas_unit_price.unwrap_or(0),
            self.ttl.unwrap_or(DEFAULT_EXPIRATION_TIME),
            ChainId::test(),
        )
    }

    pub fn sign(self) -> SignedUserTransaction {
        RawUserTransaction::new_with_default_gas_token(
            *self.sender.address(),
            self.sequence_number.expect("sequence number not set"),
            self.program.expect("transaction payload not set"),
            self.max_gas_amount.unwrap_or(gas_costs::TXN_RESERVED),
            self.gas_unit_price.unwrap_or(0),
            self.ttl.unwrap_or(DEFAULT_EXPIRATION_TIME),
            self.chain_id.unwrap_or_else(ChainId::test),
        )
        .sign(&self.sender.privkey, self.sender.pubkey)
        .unwrap()
        .into_inner()
    }

    // TODO(BobOng): e2e-test unsupport multi-agent
    // pub fn sign_multi_agent(self) -> SignedUserTransaction {
    //     let secondary_signer_addresses: Vec<AccountAddress> = self
    //         .secondary_signers
    //         .iter()
    //         .map(|signer| *signer.address())
    //         .collect();
    //     let secondary_private_keys = self
    //         .secondary_signers
    //         .iter()
    //         .map(|signer| &signer.privkey)
    //         .collect();
    //     RawUserTransaction::new_with_default_gas_token(
    //         *self.sender.address(),
    //         self.sequence_number.expect("sequence number not set"),
    //         self.program.expect("transaction payload not set"),
    //         self.max_gas_amount.unwrap_or(gas_costs::TXN_RESERVED),
    //         self.gas_unit_price.unwrap_or(0),
    //         self.ttl.unwrap_or(DEFAULT_EXPIRATION_TIME),
    //         ChainId::test(),
    //     )
    //     .sign_multi_agent(
    //         &self.sender.privkey,
    //         secondary_signer_addresses,
    //         secondary_private_keys,
    //     )
    //     .unwrap()
    //     .into_inner()
    // }
}

//---------------------------------------------------------------------------
// CoinStore resource representation
//---------------------------------------------------------------------------

/// Struct that represents an account CoinStore resource for tests.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoinStore {
    coin: u64,
    deposit_events: EventHandle,
    withdraw_events: EventHandle,
}

impl CoinStore {
    /// Create a new CoinStore
    pub fn new(coin: u64, deposit_events: EventHandle, withdraw_events: EventHandle) -> Self {
        Self {
            coin,
            deposit_events,
            withdraw_events,
        }
    }

    /// Retrieve the balance inside of this
    pub fn coin(&self) -> u64 {
        self.coin
    }

    /// Returns the Move Value for the account's CoinStore
    pub fn to_value(&self) -> Value {
        Value::struct_(Struct::pack(vec![
            Value::u64(self.coin),
            Value::struct_(Struct::pack(vec![
                Value::u64(self.withdraw_events.count()),
                Value::vector_u8(self.withdraw_events.key().to_vec()),
            ])),
            Value::struct_(Struct::pack(vec![
                Value::u64(self.deposit_events.count()),
                Value::vector_u8(self.deposit_events.key().to_vec()),
            ])),
        ]))
    }

    /// Returns the value layout for the account's CoinStore
    pub fn layout() -> MoveStructLayout {
        MoveStructLayout::new(vec![
            MoveTypeLayout::U64,
            MoveTypeLayout::Struct(MoveStructLayout::new(vec![
                MoveTypeLayout::U64,
                MoveTypeLayout::Vector(Box::new(MoveTypeLayout::U8)),
            ])),
            MoveTypeLayout::Struct(MoveStructLayout::new(vec![
                MoveTypeLayout::U64,
                MoveTypeLayout::Vector(Box::new(MoveTypeLayout::U8)),
            ])),
        ])
    }
}

//---------------------------------------------------------------------------
// Account type representation
//---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AccountRoleSpecifier {
    AptosRoot,
    TreasuryCompliance,
    DesignatedDealer,
    Validator,
    ValidatorOperator,
    ParentVASP,
    ChildVASP,
}

impl AccountRoleSpecifier {
    pub fn id(&self) -> u64 {
        match self {
            Self::AptosRoot => 0,
            Self::TreasuryCompliance => 1,
            Self::DesignatedDealer => 2,
            Self::Validator => 3,
            Self::ValidatorOperator => 4,
            Self::ParentVASP => 5,
            Self::ChildVASP => 6,
        }
    }

    pub fn layout() -> MoveStructLayout {
        MoveStructLayout::new(vec![MoveTypeLayout::U64])
    }

    pub fn to_value(&self) -> Value {
        Value::struct_(Struct::pack(vec![Value::u64(self.id())]))
    }
}

impl FromStr for AccountRoleSpecifier {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "vasp" => Ok(AccountRoleSpecifier::ParentVASP), // TODO: rename from vasp
            "validator" => Ok(AccountRoleSpecifier::Validator),
            "validator_operator" => Ok(AccountRoleSpecifier::ValidatorOperator),
            other => Err(Error::msg(format!(
                "Unrecognized account type specifier {} found.",
                other
            ))),
        }
    }
}

impl Default for AccountRoleSpecifier {
    fn default() -> Self {
        AccountRoleSpecifier::ParentVASP
    }
}

//---------------------------------------------------------------------------
// Account type resource representation
//---------------------------------------------------------------------------

/// Struct that represents an account type for testing.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountRole {
    self_address: AccountAddress,
    account_specifier: AccountRoleSpecifier,
}

impl AccountRole {
    /// Create a new AccountRole testing account.
    pub fn new(self_address: AccountAddress, account_specifier: AccountRoleSpecifier) -> Self {
        Self {
            self_address,
            account_specifier,
        }
    }

    pub fn account_specifier(&self) -> AccountRoleSpecifier {
        self.account_specifier
    }
}

/// Represents an account along with initial state about it.
///
/// `AccountData` captures the initial state needed to create accounts for tests.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountData {
    account: Account,
    withdrawal_capability: Option<WithdrawCapability>,
    key_rotation_capability: Option<KeyRotationCapability>,
    sequence_number: u64,

    coin_store: CoinStore,
    account_role: AccountRole,
}

fn new_event_handle(count: u64, address: AccountAddress) -> EventHandle {
    EventHandle::new_from_address(&address, count)
}

impl AccountData {
    /// Creates a new `AccountData` with a new account.
    ///
    /// This constructor is non-deterministic and should not be used against golden file.
    pub fn new(balance: u64, sequence_number: u64) -> Self {
        Self::with_account(
            Account::new(),
            balance,
            sequence_number,
            AccountRoleSpecifier::ParentVASP,
        )
    }

    /// Creates a new `AccountData` with a new account.
    ///
    /// Most tests will want to use this constructor.
    pub fn new_from_seed(seed: &mut KeyGen, balance: u64, sequence_number: u64) -> Self {
        Self::with_account(
            Account::new_from_seed(seed),
            balance,
            sequence_number,
            AccountRoleSpecifier::ParentVASP,
        )
    }

    /// Creates a new `AccountData` with the provided account.
    pub fn with_account(
        account: Account,
        balance: u64,
        sequence_number: u64,
        account_specifier: AccountRoleSpecifier,
    ) -> Self {
        Self::with_account_and_event_counts(
            account,
            balance,
            sequence_number,
            0,
            0,
            account_specifier,
        )
    }

    /// Creates a new `AccountData` with the provided account.
    pub fn with_keypair(
        privkey: Ed25519PrivateKey,
        pubkey: Ed25519PublicKey,
        balance: u64,
        sequence_number: u64,
        account_specifier: AccountRoleSpecifier,
    ) -> Self {
        let account = Account::with_keypair(privkey, pubkey);
        Self::with_account(account, balance, sequence_number, account_specifier)
    }

    /// Creates a new `AccountData` with custom parameters.
    pub fn with_account_and_event_counts(
        account: Account,
        balance: u64,
        sequence_number: u64,
        sent_events_count: u64,
        received_events_count: u64,
        account_specifier: AccountRoleSpecifier,
    ) -> Self {
        let addr = *account.address();
        Self {
            account_role: AccountRole::new(*account.address(), account_specifier),
            withdrawal_capability: Some(WithdrawCapability::new(addr)),
            key_rotation_capability: Some(KeyRotationCapability::new(addr)),
            account,
            coin_store: CoinStore::new(
                balance,
                new_event_handle(received_events_count, addr),
                new_event_handle(sent_events_count, addr),
            ),
            sequence_number,
        }
    }

    /// Changes the keys for this account to the provided ones.
    pub fn rotate_key(&mut self, privkey: Ed25519PrivateKey, pubkey: Ed25519PublicKey) {
        self.account.rotate_key(privkey, pubkey)
    }

    /// Returns the (Move value) layout of the Account::Account struct
    pub fn layout() -> MoveStructLayout {
        use MoveStructLayout as S;
        use MoveTypeLayout as T;

        S::new(vec![T::Vector(Box::new(T::U8)), T::U64, T::Address])
    }

    /// Returns the account role specifier
    pub fn account_role(&self) -> AccountRoleSpecifier {
        self.account_role.account_specifier()
    }

    /// Creates and returns the top-level resources to be published under the account
    pub fn to_value(&self) -> (Value, Value) {
        // TODO: publish some concept of Account
        let account = Value::struct_(Struct::pack(vec![
            // TODO: this needs to compute the auth key instead
            Value::vector_u8(AuthenticationKey::ed25519(&self.account.pubkey).to_vec()),
            Value::u64(self.sequence_number),
            Value::address(*self.address()),
        ]));
        (account, self.coin_store.to_value())
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

    pub fn transfer_event_layout() -> MoveStructLayout {
        let event_layout = MoveTypeLayout::Struct(MoveStructLayout::new(vec![
            MoveTypeLayout::U64,
            MoveTypeLayout::Vector(Box::new(MoveTypeLayout::U8)),
        ]));
        MoveStructLayout::new(vec![event_layout.clone(), event_layout])
    }

    /// Creates a writeset that contains the account data and can be patched to the storage
    /// directly.
    pub fn to_writeset(&self) -> WriteSet {
        let (account_blob, coinstore_blob) = self.to_value();
        let mut write_set = Vec::new();
        let account = account_blob
            .value_as::<Struct>()
            .unwrap()
            .simple_serialize(&AccountData::layout())
            .unwrap();
        write_set.push((
            StateKey::AccessPath(self.make_account_access_path()),
            WriteOp::Value(account),
        ));

        let balance = coinstore_blob
            .value_as::<Struct>()
            .unwrap()
            .simple_serialize(&CoinStore::layout())
            .unwrap();
        write_set.push((
            StateKey::AccessPath(self.make_coin_store_access_path()),
            WriteOp::Value(balance),
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
    pub fn balance(&self) -> u64 {
        self.coin_store.coin()
    }

    /// Returns the initial sequence number.
    pub fn sequence_number(&self) -> u64 {
        self.sequence_number
    }

    /// Returns the unique key for this sent events stream.
    pub fn sent_events_key(&self) -> &[u8] {
        self.coin_store.withdraw_events.key().as_bytes()
    }

    /// Returns the initial sent events count.
    pub fn sent_events_count(&self) -> u64 {
        self.coin_store.withdraw_events.count()
    }

    /// Returns the unique key for this received events stream.
    pub fn received_events_key(&self) -> &[u8] {
        self.coin_store.deposit_events.key().as_bytes()
    }

    /// Returns the initial received events count.
    pub fn received_events_count(&self) -> u64 {
        self.coin_store.deposit_events.count()
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FreezingBit {
    is_frozen: bool,
}

impl FreezingBit {
    pub fn layout() -> MoveStructLayout {
        MoveStructLayout::new(vec![MoveTypeLayout::Bool])
    }

    pub fn value() -> Value {
        Value::struct_(Struct::pack(vec![Value::bool(false)]))
    }
}
