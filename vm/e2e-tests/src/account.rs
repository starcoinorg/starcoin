// Copyright (c) Starcoin
// SPDX-License-Identifier: Apache-2.0

//! Test infrastructure for modeling Starcoin accounts.

use crate::gas_costs;
use anyhow::{Error, Result};
use starcoin_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey},
    keygen::KeyGen,
    ValidCryptoMaterial,
};
use starcoin_types::account::{Balance, EventHandleGenerator};
use starcoin_vm_types::{
    access_path::AccessPath,
    account_address::AccountAddress,
    account_config,
    account_config::{genesis_address, AccountResource, BalanceResource},
    event::EventHandle,
    genesis_config::ChainId,
    language_storage::StructTag,
    move_resource::MoveResource,
    state_store::state_key::StateKey,
    token::token_code::TokenCode,
    transaction::{
        authenticator::{AccountPrivateKey, AccountPublicKey, AuthenticationKey},
        Module, Package, RawUserTransaction, Script, ScriptFunction, SignedUserTransaction,
        TransactionPayload,
    },
    value::{MoveStructLayout, MoveTypeLayout},
    values::{Struct, Value},
    write_set::{WriteOp, WriteSet, WriteSetMut},
};
use std::{collections::BTreeMap, str::FromStr, sync::Arc};

// TTL is 86400s. Initial time was set to 0.
pub const DEFAULT_EXPIRATION_TIME: u64 = 40_000;

pub const STC_TOKEN_CODE_STR: &str = "0x1::STC::STC";

/// Details about a Starcoin account.
///
/// Tests will typically create a set of `Account` instances to run transactions on. This type
/// encodes the logic to operate on and verify operations on any Starcoin account.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Account {
    // addr: AccountAddress,
    // /// The current private key for this account.
    // pub privkey: Ed25519PrivateKey,
    // /// The current public key for this account.
    // pub pubkey: Ed25519PublicKey,
    pub addr: AccountAddress,
    private_key: Arc<AccountPrivateKey>,
}

impl Account {
    /// Creates a new account in memory.
    ///
    /// The account returned by this constructor is a purely logical entity, meaning that it does
    /// not automatically get added to the Starcoin store. To add an account to the store, use
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
            private_key: Arc::new(AccountPrivateKey::Single(privkey)),
            // pubkey,
        }
    }

    /// Get the Key pair
    pub fn ed25519_key_pair(&self) -> (Ed25519PublicKey, Ed25519PrivateKey) {
        (
            self.private_key.public_key().as_single().unwrap(),
            match self.private_key.as_ref() {
                AccountPrivateKey::Single(pk) => pk.clone(),
                AccountPrivateKey::Multi(pk) => pk.private_keys().get(0).unwrap().clone(),
            },
        )
    }

    /// Creates a new account with the given addr and key pair
    ///
    /// Like with [`Account::new`], the account returned by this constructor is a purely logical
    /// entity.
    pub fn new_validator(
        addr: AccountAddress,
        privkey: Ed25519PrivateKey,
        _pubkey: Ed25519PublicKey,
    ) -> Self {
        Account {
            addr,
            private_key: Arc::new(AccountPrivateKey::Single(privkey)),
            // pubkey,
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
        let (privkey, _pubkey) = KeyGen::from_os_rng().generate_keypair();
        Account {
            addr: address,
            private_key: Arc::new(AccountPrivateKey::Single(privkey)),
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

    pub fn new_testing_dd() -> Self {
        Self::new_genesis_account(
            AccountAddress::from_hex_literal("0xDD")
                .expect("Parsing valid hex literal should always succeed"),
        )
    }

    pub fn new_blessed_tc() -> Self {
        Self::new_genesis_account(
            AccountAddress::from_hex_literal("0xB1E55ED")
                .expect("Parsing valid hex literal should always succeed"),
        )
    }

    pub fn auth_key_prefix(&self) -> Vec<u8> {
        AuthenticationKey::ed25519(&self.public_key())
            .prefix()
            .to_vec()
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

    // TODO: plug in the account type
    pub fn make_access_path(&self, tag: StructTag) -> AccessPath {
        // TODO: we need a way to get the type (FatStructType) of the Account in place
        // let resource_tag = ResourceKey::new(self.addr, tag);
        AccessPath::resource_access_path(self.addr, tag)
    }

    pub fn make_account_access_path(&self) -> AccessPath {
        self.make_access_path(AccountResource::struct_tag())
    }

    /// Returns the AccessPath that describes the EventHandleGenerator resource instance.
    ///
    /// Use this to retrieve or publish the EventHandleGenerator blob.
    pub fn make_event_generator_access_path(&self) -> AccessPath {
        self.make_access_path(account_config::event_handle_generator_struct_tag())
    }

    /// Changes the keys for this account to the provided ones.
    pub fn rotate_key(&mut self, privkey: Ed25519PrivateKey, _pubkey: Ed25519PublicKey) {
        // self.privkey = privkey;
        // self.pubkey = pubkey;
        self.private_key = Arc::new(AccountPrivateKey::Single(privkey));
    }

    /// Computes the authentication key for this account, as stored on the chain.
    ///
    /// This is the same as the account's address if the keys have never been rotated.
    pub fn auth_key(&self) -> Vec<u8> {
        self.private_key.public_key().authentication_key().to_vec()
    }

    pub fn public_key(&self) -> Ed25519PublicKey {
        match self.private_key.as_ref() {
            AccountPrivateKey::Single(k) => Ed25519PublicKey::from(k),
            AccountPrivateKey::Multi(k) => k.public_key().public_keys().get(0).unwrap().clone(),
        }
    }

    pub fn private_key(&self) -> &AccountPrivateKey {
        &self.private_key
    }

    pub fn transaction(&self) -> TransactionBuilder {
        TransactionBuilder::new(self.clone())
    }

    pub fn account_keypair(&self) -> (AccountPublicKey, AccountPrivateKey) {
        let (pub_key, priv_key) = self.ed25519_key_pair();
        (
            AccountPublicKey::Single(pub_key),
            AccountPrivateKey::Single(priv_key),
        )
    }

    pub fn public_key_bytes(&self) -> Vec<u8> {
        match self.private_key.as_ref() {
            AccountPrivateKey::Single(public_key) => public_key.to_bytes().to_vec(),
            AccountPrivateKey::Multi(public_key) => public_key.to_bytes().to_vec(),
        }
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
    pub gas_currency_code: Option<String>,
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
            gas_currency_code: None,
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

    pub fn module(mut self, m: Module) -> Self {
        self.program = Some(TransactionPayload::Package(
            Package::new_with_module(m).unwrap(),
        ));
        self
    }

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

    pub fn gas_currency_code(mut self, gas_currency_code: &str) -> Self {
        self.gas_currency_code = Some(gas_currency_code.to_string());
        self
    }

    pub fn ttl(mut self, ttl: u64) -> Self {
        self.ttl = Some(ttl);
        self
    }

    pub fn raw(self) -> RawUserTransaction {
        RawUserTransaction::new(
            *self.sender.address(),
            self.sequence_number.expect("sequence number not set"),
            self.program.expect("transaction payload not set"),
            self.max_gas_amount.unwrap_or(gas_costs::TXN_RESERVED),
            self.gas_unit_price.unwrap_or(0),
            self.ttl.unwrap_or(DEFAULT_EXPIRATION_TIME),
            ChainId::test(),
            self.gas_currency_code
                .unwrap_or_else(|| STC_TOKEN_CODE_STR.to_string()),
        )
    }

    pub fn sign(self) -> SignedUserTransaction {
        let (public_key, private_key) = self.sender.ed25519_key_pair();
        RawUserTransaction::new_with_default_gas_token(
            *self.sender.address(),
            self.sequence_number.expect("sequence number not set"),
            self.program.expect("transaction payload not set"),
            self.max_gas_amount.unwrap_or(gas_costs::TXN_RESERVED),
            self.gas_unit_price.unwrap_or(0),
            self.ttl.unwrap_or(DEFAULT_EXPIRATION_TIME),
            self.chain_id.unwrap_or_else(ChainId::test),
        )
        .sign(&private_key, public_key)
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
//
// //---------------------------------------------------------------------------
// // CoinStore resource represenation
// //---------------------------------------------------------------------------
//
// /// Struct that represents an account CoinStore resource for tests.
// #[derive(Clone, Debug, Eq, PartialEq)]
// pub struct CoinStore {
//     coin: u64,
//     deposit_events: EventHandle,
//     withdraw_events: EventHandle,
// }
//
// impl CoinStore {
//     /// Create a new CoinStore
//     pub fn new(coin: u64, deposit_events: EventHandle, withdraw_events: EventHandle) -> Self {
//         Self {
//             coin,
//             deposit_events,
//             withdraw_events,
//         }
//     }
//
//     /// Retrieve the balance inside of this
//     pub fn coin(&self) -> u64 {
//         self.coin
//     }
//
//     /// Returns the Move Value for the account's CoinStore
//     pub fn to_value(&self) -> Value {
//         Value::struct_(Struct::pack(vec![
//             Value::u64(self.coin),
//             Value::struct_(Struct::pack(vec![
//                 Value::u64(self.withdraw_events.count()),
//                 Value::vector_u8(self.withdraw_events.key().to_vec()),
//             ])),
//             Value::struct_(Struct::pack(vec![
//                 Value::u64(self.deposit_events.count()),
//                 Value::vector_u8(self.deposit_events.key().to_vec()),
//             ])),
//         ]))
//     }
//
//     /// Returns the value layout for the account's CoinStore
//     pub fn layout() -> MoveStructLayout {
//         MoveStructLayout::new(vec![
//             MoveTypeLayout::U64,
//             MoveTypeLayout::Struct(MoveStructLayout::new(vec![
//                 MoveTypeLayout::U64,
//                 MoveTypeLayout::Vector(Box::new(MoveTypeLayout::U8)),
//             ])),
//             MoveTypeLayout::Struct(MoveStructLayout::new(vec![
//                 MoveTypeLayout::U64,
//                 MoveTypeLayout::Vector(Box::new(MoveTypeLayout::U8)),
//             ])),
//         ])
//     }
// }

//---------------------------------------------------------------------------
// Account type represenation
//---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AccountRoleSpecifier {
    Root,
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
            Self::Root => 0,
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
// Account type resource represenation
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
    // account: Account,
    // withdrawal_capability: Option<WithdrawCapability>,
    // key_rotation_capability: Option<KeyRotationCapability>,
    // sequence_number: u64,
    //
    // coin_store: CoinStore,
    // account_role: AccountRole,
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

fn new_event_handle(count: u64, address: AccountAddress) -> EventHandle {
    EventHandle::new_from_address(&address, count)
}

impl AccountData {
    /// Creates a new `AccountData` with a new account.
    ///
    /// This constructor is non-deterministic and should not be used against golden file.
    pub fn new(balance: u128, sequence_number: u64) -> Self {
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
    pub fn new_from_seed(seed: &mut KeyGen, balance: u128, sequence_number: u64) -> Self {
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
        balance: u128,
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
        balance: u128,
        sequence_number: u64,
        account_specifier: AccountRoleSpecifier,
    ) -> Self {
        let account = Account::with_keypair(privkey, pubkey);
        Self::with_account(account, balance, sequence_number, account_specifier)
    }

    /// Creates a new `AccountData` with custom parameters.
    pub fn with_account_and_event_counts(
        account: Account,
        balance: u128,
        sequence_number: u64,
        sent_events_count: u64,
        received_events_count: u64,
        _account_specifier: AccountRoleSpecifier,
    ) -> Self {
        let addr = *account.address();
        let mut balances_map = BTreeMap::new();
        balances_map.insert(STC_TOKEN_CODE_STR.to_string(), Balance::new(balance));
        Self {
            event_generator: EventHandleGenerator::new_with_event_count(addr, sent_events_count),
            account,
            balances: balances_map,
            sequence_number,
            key_rotation_capability: Some(KeyRotationCapability::new(addr)),
            withdrawal_capability: Some(WithdrawCapability::new(addr)),
            withdraw_events: EventHandle::new_from_address(&addr, sent_events_count),
            deposit_events: EventHandle::new_from_address(&addr, received_events_count),
            accept_token_events: EventHandle::new_from_address(&addr, 0),
        }
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
            T::Struct(Self::event_handle_layout()),
            T::U64,
        ])
    }

    /// Returns the account role specifier
    pub fn account_role(&self) -> AccountRoleSpecifier {
        //self.account_role.account_specifier()
        AccountRoleSpecifier::ParentVASP
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

    /// Returns the AccessPath that describes the Account's Balance resource instance.
    ///
    /// Use this to retrieve or publish the Account's Balance blob.
    pub fn make_balance_access_path(&self, token_code: &str) -> AccessPath {
        self.account.make_balance_access_path(token_code)
    }

    pub fn make_event_generator_access_path(&self) -> AccessPath {
        self.account.make_event_generator_access_path()
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
        let (account_blob, balance_blobs, event_generator_blob) = self.to_value();
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
        for (code, balance_blob) in balance_blobs.into_iter() {
            let balance = balance_blob
                .value_as::<Struct>()
                .unwrap()
                .simple_serialize(&Balance::layout())
                .unwrap();
            write_set.push((
                StateKey::AccessPath(self.make_balance_access_path(code.as_str())),
                WriteOp::Value(balance),
            ));
        }

        let event_generator = event_generator_blob
            .value_as::<Struct>()
            .unwrap()
            .simple_serialize(&EventHandleGenerator::layout())
            .unwrap();
        write_set.push((
            StateKey::AccessPath(self.make_event_generator_access_path()),
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
    pub fn balance(&self) -> u128 {
        self.balances
            .get(STC_TOKEN_CODE_STR)
            .expect("get balance by currency_code fail")
            .token()
    }

    /// Returns the initial sequence number.
    pub fn sequence_number(&self) -> u64 {
        self.sequence_number
    }

    /// Returns the unique key for this sent events stream.
    pub fn sent_events_key(&self) -> &[u8] {
        self.withdraw_events.key().as_bytes()
    }

    /// Returns the initial sent events count.
    pub fn sent_events_count(&self) -> u64 {
        self.withdraw_events.count()
    }

    /// Returns the unique key for this received events stream.
    pub fn received_events_key(&self) -> &[u8] {
        self.deposit_events.key().as_bytes()
    }

    /// Returns the initial received events count.
    pub fn received_events_count(&self) -> u64 {
        self.deposit_events.count()
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
