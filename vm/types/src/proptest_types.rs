// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
#![allow(clippy::unit_arg)]
use crate::account_address::AccountAddress;
use crate::block_metadata::BlockMetadata;
use crate::event::EventHandle;
use crate::genesis_config::ChainId;
use crate::identifier::Identifier;
use crate::language_storage::ModuleId;
use crate::time::{MockTimeService, TimeService};
use crate::transaction::authenticator::AuthenticationKey;
use crate::transaction::{
    Module, Package, RawUserTransaction, Script, ScriptFunction, SignatureCheckedTransaction,
    SignedUserTransaction, TransactionPayload,
};
use crate::transaction_argument::convert_txn_args;
use crate::transaction_argument::TransactionArgument;
use crate::{account_address, account_config};
use anyhow::Result;
use move_core_types::language_storage::TypeTag;
use proptest::collection::SizeRange;
use proptest::sample::Index as PropIndex;
use proptest::{collection::vec, prelude::*};
use proptest_derive::Arbitrary;
use starcoin_crypto::ed25519::{Ed25519PrivateKey, Ed25519PublicKey};
use starcoin_crypto::multi_ed25519::multi_shard::MultiEd25519KeyShard;
use starcoin_crypto::multi_ed25519::MultiEd25519PublicKey;
use starcoin_crypto::test_utils::KeyPair;
use starcoin_crypto::{ed25519, HashValue, SigningKey};
use std::ops::Deref;
use std::sync::Arc;
use vm::CompiledModule;

/// Wrapper for `proptest`'s [`Index`][proptest::sample::Index] that allows `AsRef` to work.
///
/// There is no blanket `impl<T> AsRef<T> for T`, so `&[PropIndex]` doesn't work with
/// `&[impl AsRef<PropIndex>]` (unless an impl gets added upstream). `Index` does.
#[derive(Arbitrary, Clone, Copy, Debug)]
pub struct Index(PropIndex);

impl AsRef<PropIndex> for Index {
    fn as_ref(&self) -> &PropIndex {
        &self.0
    }
}

impl Deref for Index {
    type Target = PropIndex;

    fn deref(&self) -> &PropIndex {
        &self.0
    }
}

/// A private key and public key pair holder for test
#[derive(Debug, Clone)]
pub enum KeyPairHolder {
    Ed25519(Arc<Ed25519PrivateKey>, Ed25519PublicKey),
    MultiEd25519(Arc<MultiEd25519KeyShard>, MultiEd25519PublicKey),
}

impl KeyPairHolder {
    pub fn new(private_key: Arc<Ed25519PrivateKey>, public_key: Ed25519PublicKey) -> Self {
        Self::Ed25519(private_key, public_key)
    }

    pub fn new_multi(
        private_key: Arc<MultiEd25519KeyShard>,
        public_key: MultiEd25519PublicKey,
    ) -> Self {
        Self::MultiEd25519(private_key, public_key)
    }

    pub fn sign_txn(&self, txn: RawUserTransaction) -> Result<SignedUserTransaction> {
        Ok(match self {
            Self::Ed25519(private_key, public_key) => {
                let signature = private_key.sign(&txn);
                SignedUserTransaction::ed25519(txn, public_key.clone(), signature)
            }
            Self::MultiEd25519(private_key, public_key) => {
                let signature = private_key.sign(&txn);
                SignedUserTransaction::multi_ed25519(txn, public_key.clone(), signature.into())
            }
        })
    }

    pub fn auth_key(&self) -> AuthenticationKey {
        match self {
            Self::Ed25519(_, public_key) => AuthenticationKey::ed25519(public_key),
            Self::MultiEd25519(_, public_key) => AuthenticationKey::multi_ed25519(public_key),
        }
    }
}

#[derive(Clone, Debug)]
pub struct AccountInfo {
    address: AccountAddress,
    key_pair: KeyPairHolder,
    sequence_number: u64,
    #[allow(unused)]
    sent_event_handle: EventHandle,
    #[allow(unused)]
    received_event_handle: EventHandle,
}

impl AccountInfo {
    pub fn new(private_key: Ed25519PrivateKey, public_key: Ed25519PublicKey) -> Self {
        let address = account_address::from_public_key(&public_key);
        Self {
            address,
            key_pair: KeyPairHolder::new(Arc::new(private_key), public_key),
            sequence_number: 0,
            sent_event_handle: EventHandle::new_from_address(&address, 0),
            received_event_handle: EventHandle::new_from_address(&address, 1),
        }
    }
    pub fn new_with_address(
        address: AccountAddress,
        private_key: Ed25519PrivateKey,
        public_key: Ed25519PublicKey,
    ) -> Self {
        Self {
            address,
            key_pair: KeyPairHolder::new(Arc::new(private_key), public_key),
            sequence_number: 0,
            sent_event_handle: EventHandle::new_from_address(&address, 0),
            received_event_handle: EventHandle::new_from_address(&address, 1),
        }
    }

    pub fn new_multi(
        address: AccountAddress,
        private_key: Arc<MultiEd25519KeyShard>,
        public_key: MultiEd25519PublicKey,
    ) -> Self {
        Self {
            address,
            key_pair: KeyPairHolder::new_multi(private_key, public_key),
            sequence_number: 0,
            sent_event_handle: EventHandle::new_from_address(&address, 0),
            received_event_handle: EventHandle::new_from_address(&address, 1),
        }
    }
}

#[derive(Clone, Debug)]
pub struct AccountInfoUniverse {
    accounts: Vec<AccountInfo>,
    #[allow(unused)]
    epoch: u64,
    chain_id: ChainId,
    time_service: Arc<dyn TimeService>,
}

impl AccountInfoUniverse {
    fn new(key_pairs: Vec<(Ed25519PrivateKey, Ed25519PublicKey)>, epoch: u64) -> Self {
        let accounts = key_pairs
            .into_iter()
            .map(|(private_key, public_key)| AccountInfo::new(private_key, public_key))
            .collect();
        let time_service = MockTimeService::new();
        Self {
            accounts,
            epoch,
            chain_id: ChainId::test(),
            time_service: Arc::new(time_service),
        }
    }

    pub fn default() -> Result<Self> {
        let (private_key, public_key) = starcoin_crypto::ed25519::genesis_key_pair();
        let account = AccountInfo::new(private_key, public_key);
        Ok(Self {
            accounts: vec![account],
            epoch: 0,
            chain_id: ChainId::test(),
            time_service: Arc::new(MockTimeService::new()),
        })
    }

    fn get_account_info(&self, account_index: Index) -> &AccountInfo {
        account_index.get(&self.accounts)
    }

    fn get_account_info_mut(&mut self, account_index: Index) -> &mut AccountInfo {
        account_index.get_mut(self.accounts.as_mut_slice())
    }

    fn _get_epoch(&self) -> u64 {
        self.epoch
    }

    fn _get_and_bump_epoch(&mut self) -> u64 {
        let epoch = self.epoch;
        self.epoch += 1;
        epoch
    }

    pub fn time_service(&self) -> &dyn TimeService {
        self.time_service.as_ref()
    }

    pub fn chain_id(&self) -> ChainId {
        self.chain_id
    }
}

impl Arbitrary for AccountInfoUniverse {
    type Parameters = usize;
    fn arbitrary() -> Self::Strategy {
        unimplemented!("Size of the universe must be provided explicitly (use any_with instead).")
    }

    fn arbitrary_with(num_accounts: Self::Parameters) -> Self::Strategy {
        vec(ed25519::keypair_strategy(), num_accounts)
            .prop_map(|kps| {
                let kps: Vec<_> = kps
                    .into_iter()
                    .map(|k| (k.private_key, k.public_key))
                    .collect();
                AccountInfoUniverse::new(kps, /* epoch = */ 0)
            })
            .boxed()
    }

    type Strategy = BoxedStrategy<Self>;
}

#[derive(Arbitrary, Debug)]
pub struct RawUserTransactionGen {
    payload: TransactionPayload,
    #[allow(unused)]
    max_gas_amount: u64,
    #[allow(unused)]
    gas_unit_price: u64,
    #[allow(unused)]
    gas_currency_code: String,
    #[allow(unused)]
    expiration_time_secs: u64,
}

impl RawUserTransactionGen {
    pub fn materialize(
        self,
        sender_index: Index,
        universe: &mut AccountInfoUniverse,
        expired_time: u64,
        payload: Option<TransactionPayload>,
    ) -> RawUserTransaction {
        let mut sender_info = universe.get_account_info_mut(sender_index);

        let sequence_number = sender_info.sequence_number;
        let temp_payload = payload.unwrap_or(self.payload);

        sender_info.sequence_number += 1;
        RawUserTransaction::new_with_default_gas_token(
            sender_info.address,
            sequence_number,
            temp_payload,
            20000,
            1,
            expired_time,
            ChainId::test(),
        )
    }
}

impl RawUserTransaction {
    fn strategy_impl(
        address_strategy: impl Strategy<Value = AccountAddress>,
        payload_strategy: impl Strategy<Value = TransactionPayload>,
    ) -> impl Strategy<Value = Self> {
        // XXX what other constraints do these need to obey?
        (
            address_strategy,
            any::<u64>(),
            payload_strategy,
            Just(20000u64),
            Just(1u64),
            any::<u64>(),
        )
            .prop_map(
                |(
                    sender,
                    sequence_number,
                    payload,
                    max_gas_amount,
                    gas_unit_price,
                    expiration_time_secs,
                )| {
                    RawUserTransaction::new_with_default_gas_token(
                        sender,
                        sequence_number,
                        payload,
                        max_gas_amount,
                        gas_unit_price,
                        expiration_time_secs,
                        ChainId::test(),
                    )
                },
            )
    }
}

impl Arbitrary for RawUserTransaction {
    type Parameters = ();
    fn arbitrary_with(_args: ()) -> Self::Strategy {
        Self::strategy_impl(
            Just(account_config::association_address()),
            any::<TransactionPayload>(),
        )
        .boxed()
    }

    type Strategy = BoxedStrategy<Self>;
}

impl SignatureCheckedTransaction {
    // This isn't an Arbitrary impl because this doesn't generate *any* possible SignedTransaction,
    // just one kind of them.
    pub fn script_strategy(
        keypair_strategy: impl Strategy<Value = KeyPair<Ed25519PrivateKey, Ed25519PublicKey>>,
    ) -> impl Strategy<Value = Self> {
        Self::strategy_impl(keypair_strategy, TransactionPayload::script_strategy())
    }

    pub fn package_strategy(
        keypair_strategy: impl Strategy<Value = KeyPair<Ed25519PrivateKey, Ed25519PublicKey>>,
    ) -> impl Strategy<Value = Self> {
        Self::strategy_impl(keypair_strategy, TransactionPayload::package_strategy())
    }

    fn strategy_impl(
        keypair_strategy: impl Strategy<Value = KeyPair<Ed25519PrivateKey, Ed25519PublicKey>>,
        payload_strategy: impl Strategy<Value = TransactionPayload>,
    ) -> impl Strategy<Value = Self> {
        (keypair_strategy, payload_strategy)
            .prop_flat_map(|(keypair, payload)| {
                // let address = account_address::from_public_key(&keypair.public_key);
                (
                    Just(keypair),
                    RawUserTransaction::strategy_impl(
                        Just(account_config::association_address()),
                        Just(payload),
                    ),
                )
            })
            .prop_flat_map(|(keypair, raw_txn)| {
                let sign_txn = raw_txn
                    .sign(&keypair.private_key, keypair.public_key.clone())
                    .unwrap();
                prop_oneof![Just(sign_txn),]
            })
    }
}

#[derive(Arbitrary, Debug)]
pub struct SignatureCheckedTransactionGen {
    raw_transaction_gen: RawUserTransactionGen,
}

impl SignatureCheckedTransactionGen {
    pub fn materialize(
        self,
        sender_index: Index,
        universe: &mut AccountInfoUniverse,
        expired_time: u64,
        payload: Option<TransactionPayload>,
    ) -> SignatureCheckedTransaction {
        let raw_txn =
            self.raw_transaction_gen
                .materialize(sender_index, universe, expired_time, payload);
        let account_info = universe.get_account_info(sender_index);
        let txn = account_info
            .key_pair
            .sign_txn(raw_txn)
            .expect("Signing raw transaction should work.");
        txn.check_signature().expect("Check signature should ok")
    }
}

impl Arbitrary for SignatureCheckedTransaction {
    type Parameters = ();
    fn arbitrary_with(_args: ()) -> Self::Strategy {
        Self::strategy_impl(ed25519::keypair_strategy(), any::<TransactionPayload>()).boxed()
    }

    type Strategy = BoxedStrategy<Self>;
}
/// This `Arbitrary` impl only generates valid signed transactions. TODO: maybe add invalid ones?
impl Arbitrary for SignedUserTransaction {
    type Parameters = ();
    fn arbitrary_with(_args: ()) -> Self::Strategy {
        any::<SignatureCheckedTransaction>()
            .prop_map(|txn| txn.into_inner())
            .boxed()
    }

    type Strategy = BoxedStrategy<Self>;
}

impl TransactionPayload {
    pub fn script_strategy() -> impl Strategy<Value = Self> {
        any::<Script>().prop_map(TransactionPayload::Script)
    }

    pub fn package_strategy() -> impl Strategy<Value = Self> {
        any::<Package>().prop_map(TransactionPayload::Package)
    }
}

impl Arbitrary for TransactionPayload {
    type Parameters = ();
    fn arbitrary_with(_args: ()) -> Self::Strategy {
        // Most transactions in practice will be programs, but other parts of the system should
        // at least not choke on write set strategies so introduce them with decent probability.
        // The figures below are probability weights.
        prop_oneof![
            4 => Self::script_strategy(),
            1 => Self::package_strategy(),
        ]
        .boxed()
    }

    type Strategy = BoxedStrategy<Self>;
}

impl Arbitrary for Script {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: ()) -> Self::Strategy {
        // XXX This should eventually be an actually valid program, maybe?
        // The vector sizes are picked out of thin air.
        (
            vec(any::<u8>(), 0..100),
            vec(any::<TypeTag>(), 0..4),
            vec(any::<Vec<u8>>(), 0..10),
        )
            .prop_map(|(code, ty_args, args)| Script::new(code, ty_args, args))
            .boxed()
    }
}

impl Arbitrary for ScriptFunction {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: ()) -> Self::Strategy {
        // XXX This should eventually be an actually valid program, maybe?
        // The vector sizes are picked out of thin air.
        (
            any::<ModuleId>(),
            any::<Identifier>(),
            vec(any::<TypeTag>(), 0..4),
            vec(any::<TransactionArgument>(), 0..10),
        )
            .prop_map(|(module, function, ty_args, args)| {
                ScriptFunction::new(module, function, ty_args, convert_txn_args(&args))
            })
            .boxed()
    }
}

impl Arbitrary for Module {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: ()) -> Self::Strategy {
        // XXX How should we generate random modules?
        // The vector sizes are picked out of thin air.
        vec(any::<u8>(), 0..100).prop_map(Module::new).boxed()
    }
}

impl Package {
    fn strategy_impl(
        compiled_module_strategy: impl Strategy<Value = CompiledModule>,
        script_strategy: impl Strategy<Value = ScriptFunction>,
    ) -> impl Strategy<Value = Self> {
        (compiled_module_strategy, script_strategy).prop_map(|(compile_module, script)| {
            let mut vec_bytes: Vec<u8> = vec![];
            compile_module
                .serialize(&mut vec_bytes)
                .expect("compile module serialize  must success");
            let first_module = Module::new(vec_bytes);
            let module_vec = vec![first_module];
            Package::new(module_vec, Some(script)).expect("package init error")
        })
    }
}
impl Arbitrary for Package {
    type Parameters = ();
    fn arbitrary_with(_args: ()) -> Self::Strategy {
        Self::strategy_impl(
            CompiledModule::valid_strategy(20),
            // CompiledModuleStrategyGen::new(20).generate(),
            any::<ScriptFunction>(),
        )
        .boxed()
    }

    type Strategy = BoxedStrategy<Self>;
}

impl Arbitrary for BlockMetadata {
    type Parameters = SizeRange;
    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        let public_key_strategy =
            ed25519::keypair_strategy().prop_map(|key_pair| key_pair.public_key);
        (
            any::<HashValue>(),
            any::<u64>(),
            any::<AccountAddress>(),
            public_key_strategy,
            any::<u64>(),
            any::<u64>(),
            any::<u64>(),
        )
            .prop_map(
                |(
                    parent_hash,
                    timestamp,
                    addresses,
                    author_public_key,
                    uncles,
                    number,
                    parent_gas_used,
                )| {
                    BlockMetadata::new(
                        parent_hash,
                        timestamp,
                        addresses,
                        Some(AuthenticationKey::ed25519(&author_public_key)),
                        uncles,
                        number,
                        ChainId::test(),
                        parent_gas_used,
                    )
                },
            )
            .boxed()
    }

    type Strategy = BoxedStrategy<Self>;
}
