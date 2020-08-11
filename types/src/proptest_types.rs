// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_address;
use crate::account_address::AccountAddress;
use crate::chain_config::ChainId;
use crate::event::EventHandle;
use crate::transaction::{RawUserTransaction, SignedUserTransaction, TransactionPayload};
use libra_types::proptest_types::RawTransactionGen;
use libra_types::transaction::WriteSetPayload;
use proptest::arbitrary::{any, Arbitrary};
use proptest::collection::vec;
use proptest::strategy::{BoxedStrategy, Strategy};
use starcoin_crypto::ed25519;
use starcoin_crypto::ed25519::{Ed25519PrivateKey, Ed25519PublicKey};
use starcoin_proptest_helpers::Index;
use starcoin_vm_types::transaction::SignatureCheckedTransaction;

#[derive(Debug)]
struct AccountInfo {
    address: AccountAddress,
    private_key: Ed25519PrivateKey,
    public_key: Ed25519PublicKey,
    sequence_number: u64,
    sent_event_handle: EventHandle,
    received_event_handle: EventHandle,
}

impl AccountInfo {
    pub fn new(private_key: Ed25519PrivateKey, public_key: Ed25519PublicKey) -> Self {
        let address = account_address::from_public_key(&public_key);
        Self {
            address,
            private_key,
            public_key,
            sequence_number: 0,
            sent_event_handle: EventHandle::new_from_address(&address, 0),
            received_event_handle: EventHandle::new_from_address(&address, 1),
        }
    }
}

#[derive(Debug)]
pub struct AccountInfoUniverse {
    accounts: Vec<AccountInfo>,
    epoch: u64,
}

impl AccountInfoUniverse {
    fn new(key_pairs: Vec<(Ed25519PrivateKey, Ed25519PublicKey)>, epoch: u64) -> Self {
        let accounts = key_pairs
            .into_iter()
            .map(|(private_key, public_key)| AccountInfo::new(private_key, public_key))
            .collect();

        Self { accounts, epoch }
    }

    fn get_account_info(&self, account_index: Index) -> &AccountInfo {
        account_index.get(&self.accounts)
    }

    fn get_account_info_mut(&mut self, account_index: Index) -> &mut AccountInfo {
        account_index.get_mut(self.accounts.as_mut_slice())
    }

    fn get_epoch(&self) -> u64 {
        self.epoch
    }

    fn get_and_bump_epoch(&mut self) -> u64 {
        let epoch = self.epoch;
        self.epoch += 1;
        epoch
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
pub struct BlockBodyGen {}

#[derive(Arbitrary, Debug)]
pub struct SignedUserTransactionGen {}
#[derive(Arbitrary, Debug)]
pub struct RawUserTransactionGen {
    payload: TransactionPayload,
    max_gas_amount: u64,
    gas_unit_price: u64,
    gas_currency_code: String,
    expiration_time_secs: u64,
}

impl RawUserTransactionGen {
    pub fn materialize(
        self,
        sender_index: Index,
        universe: &mut AccountInfoUniverse,
    ) -> RawUserTransaction {
        let mut sender_info = universe.get_account_info_mut(sender_index);

        let sequence_number = sender_info.sequence_number;
        sender_info.sequence_number += 1;

        new_raw_transaction(
            sender_info.address,
            sequence_number,
            self.payload,
            self.max_gas_amount,
            self.gas_unit_price,
            self.expiration_time_secs,
        )
    }
}

impl RawUserTransaction {
    fn strategy_impl(
        address_strategy: impl Strategy<Value = AccountAddress>,
        payload_strategy: impl Strategy<Value = TransactionPayload>,
        gas_currency_code_strategy: impl Strategy<Value = String>,
    ) -> impl Strategy<Value = Self> {
        // XXX what other constraints do these need to obey?
        (
            address_strategy,
            any::<u64>(),
            payload_strategy,
            any::<u64>(),
            any::<u64>(),
            gas_currency_code_strategy,
            any::<u64>(),
        )
            .prop_map(
                |(
                    sender,
                    sequence_number,
                    payload,
                    max_gas_amount,
                    gas_unit_price,
                    gas_currency_code,
                    expiration_time_secs,
                )| {
                    new_raw_transaction(
                        sender,
                        sequence_number,
                        payload,
                        max_gas_amount,
                        gas_unit_price,
                        expiration_time_secs,
                    )
                },
            )
    }
}

fn new_raw_transaction(
    sender: AccountAddress,
    sequence_number: u64,
    payload: TransactionPayload,
    max_gas_amount: u64,
    gas_unit_price: u64,
    expiration_time_secs: u64,
) -> RawUserTransaction {
    let chain_id = ChainId::test();
    match payload {
        TransactionPayload::Module(module) => RawUserTransaction::new_module(
            sender,
            sequence_number,
            module,
            max_gas_amount,
            gas_unit_price,
            expiration_time_secs,
            chain_id,
        ),
        TransactionPayload::Script(script) => RawUserTransaction::new_script(
            sender,
            sequence_number,
            script,
            max_gas_amount,
            gas_unit_price,
            expiration_time_secs,
            chain_id,
        ),
        TransactionPayload::WriteSet(WriteSetPayload::Direct(write_set)) => {
            // It's a bit unfortunate that max_gas_amount etc is generated but
            // not used, but it isn't a huge deal.
            RawUserTransaction::new_change_set(sender, sequence_number, write_set, chain_id)
        }
        TransactionPayload::WriteSet(WriteSetPayload::Script {
            execute_as: signer,
            script,
        }) => RawUserTransaction::new_writeset_script(
            sender,
            sequence_number,
            script,
            signer,
            chain_id,
        ),
    }
}

impl Arbitrary for RawUserTransaction {
    type Parameters = ();
    fn arbitrary_with(_args: ()) -> Self::Strategy {
        Self::strategy_impl(
            any::<AccountAddress>(),
            any::<TransactionPayload>(),
            any::<String>(),
        )
        .boxed()
    }

    type Strategy = BoxedStrategy<Self>;
}

#[derive(Arbitrary, Debug)]
pub struct SignatureCheckedTransactionGen {
    raw_transaction_gen: SignedUserTransactionGen,
}

impl SignatureCheckedTransactionGen {
    pub fn materialize(
        self,
        sender_index: Index,
        universe: &mut AccountInfoUniverse,
    ) -> SignatureCheckedTransaction {
        let raw_txn = self.raw_transaction_gen.materialize(sender_index, universe);
        let account_info = universe.get_account_info(sender_index);
        raw_txn
            .sign(&account_info.private_key, account_info.public_key.clone())
            .expect("Signing raw transaction should work.")
    }
}

impl Arbitrary for SignatureCheckedTransaction {
    type Parameters = ();
    fn arbitrary_with(_args: ()) -> Self::Strategy {
        Self::strategy_impl(
            ed25519::keypair_strategy(),
            any::<TransactionPayload>(),
            any::<String>(),
        )
        .boxed()
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
