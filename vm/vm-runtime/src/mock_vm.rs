// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{account::AccountData, chain_state::StateStore};
use anyhow::{Error, Result};
use config::VMConfig;
use crypto::{ed25519::compat, ed25519::*, hash::CryptoHash, traits::SigningKey, HashValue};
use logger::prelude::*;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::convert::TryInto;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use traits::{ChainState, ChainStateReader, ChainStateWriter};
use types::{
    access_path::AccessPath,
    account_address::{AccountAddress, ADDRESS_LENGTH},
    account_config::{account_struct_tag, AccountResource},
    account_state::AccountState,
    contract_event::ContractEvent,
    language_storage::{ModuleId, StructTag, TypeTag},
    transaction::{
        RawUserTransaction, Script, SignedUserTransaction, Transaction, TransactionArgument,
        TransactionOutput, TransactionPayload, TransactionStatus,
    },
    vm_error::{StatusCode, VMStatus},
    write_set::{WriteOp, WriteSet, WriteSetMut},
};

enum MockTransaction {
    Mint {
        sender: AccountAddress,
        amount: u64,
    },
    Payment {
        sender: AccountAddress,
        recipient: AccountAddress,
        amount: u64,
    },
}

pub static KEEP_STATUS: Lazy<TransactionStatus> =
    Lazy::new(|| TransactionStatus::Keep(VMStatus::new(StatusCode::EXECUTED)));

// We use 10 as the assertion error code for insufficient balance within the Libra coin contract.
pub static DISCARD_STATUS: Lazy<TransactionStatus> = Lazy::new(|| {
    TransactionStatus::Discard(VMStatus::new(StatusCode::ABORTED).with_sub_status(10))
});

#[derive(Clone)]
pub struct MockVM {
    config: VMConfig,
}

impl MockVM {
    pub fn new(config: &VMConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }

    pub fn create_account(
        &self,
        account_address: AccountAddress,
        chain_state: &dyn ChainState,
    ) -> Result<()> {
        let mut state_store = StateStore::new(chain_state);
        state_store.create_account(account_address)
    }

    pub fn execute_transaction(
        &mut self,
        chain_state: &dyn ChainState,
        txn: Transaction,
    ) -> Result<TransactionOutput> {
        let mut state_store = StateStore::new(chain_state);
        let mut output;

        match txn {
            Transaction::UserTransaction(txn) => match decode_transaction(&txn) {
                MockTransaction::Mint { sender, amount } => {
                    let access_path = AccessPath::new_for_account(sender);
                    let account_resource: AccountResource = state_store
                        .get_from_statedb(&access_path)?
                        .unwrap()
                        .try_into()?;
                    assert_eq!(0, account_resource.balance(), "balance error");
                    let new_account_resource = AccountResource::new(
                        amount,
                        1,
                        account_resource.authentication_key().clone(),
                    );
                    state_store.set(access_path, new_account_resource.try_into()?);
                    output = TransactionOutput::new(vec![], 0, KEEP_STATUS.clone());
                }
                MockTransaction::Payment {
                    sender,
                    recipient,
                    amount,
                } => {
                    let access_path_sender = AccessPath::new_for_account(sender);
                    let access_path_receiver = AccessPath::new_for_account(recipient);

                    let account_resource_sender: AccountResource = state_store
                        .get_from_statedb(&access_path_sender)?
                        .unwrap()
                        .try_into()?;
                    let account_resource_receiver: AccountResource = state_store
                        .get_from_statedb(&access_path_receiver)?
                        .unwrap()
                        .try_into()?;

                    let balance_sender = account_resource_sender.balance();
                    let balance_receiver = account_resource_receiver.balance();

                    if balance_sender < amount {
                        output = TransactionOutput::new(vec![], 0, DISCARD_STATUS.clone());
                    } else {
                        let new_account_resource_sender = AccountResource::new(
                            balance_sender - amount,
                            account_resource_sender.sequence_number() + 1,
                            account_resource_sender.authentication_key().clone(),
                        );
                        let new_account_resource_receiver = AccountResource::new(
                            balance_receiver + amount,
                            account_resource_sender.sequence_number(),
                            account_resource_receiver.authentication_key().clone(),
                        );
                        state_store
                            .set(access_path_sender, new_account_resource_sender.try_into()?);
                        state_store.set(
                            access_path_receiver,
                            new_account_resource_receiver.try_into()?,
                        );
                        output = TransactionOutput::new(
                            vec![],
                            0,
                            TransactionStatus::Keep(VMStatus::new(StatusCode::EXECUTED)),
                        );
                    }
                }
            },
            Transaction::BlockMetadata(block_metadata) => {
                let (id, timestamp, author) = block_metadata.into_inner().unwrap();
                let access_path = AccessPath::new_for_account(author);
                let account_resource: AccountResource = state_store
                    .get_from_statedb(&access_path)
                    .and_then(|blob| match blob {
                        Some(blob) => Ok(blob),
                        None => {
                            state_store.create_account(author)?;
                            Ok(state_store
                                .get_from_statedb(&access_path)?
                                .expect("account resource must exist."))
                        }
                    })
                    .and_then(|blob| blob.try_into())?;

                let new_account_resource = AccountResource::new(
                    account_resource.balance() + 50_00000000,
                    account_resource.sequence_number(),
                    account_resource.authentication_key().clone(),
                );
                state_store.set(access_path, new_account_resource.try_into()?);
                output = TransactionOutput::new(vec![], 0, KEEP_STATUS.clone());
            }
            Transaction::StateSet(state_set) => {
                let result_status = match chain_state.apply(state_set) {
                    Ok(_) => KEEP_STATUS.clone(),
                    Err(_) => DISCARD_STATUS.clone(),
                };
                output = TransactionOutput::new(vec![], 0, result_status)
            }
        }
        Ok(output)
    }
}

fn read_balance(
    output_cache: &HashMap<AccessPath, u64>,
    chain_state: &dyn ChainState,
    account: AccountAddress,
) -> u64 {
    let balance_access_path = balance_ap(account);
    match output_cache.get(&balance_access_path) {
        Some(balance) => *balance,
        None => read_balance_from_storage(chain_state, &balance_access_path),
    }
}

fn read_seqnum(
    output_cache: &HashMap<AccessPath, u64>,
    chain_state: &dyn ChainState,
    account: AccountAddress,
) -> u64 {
    let seqnum_access_path = seqnum_ap(account);
    match output_cache.get(&seqnum_access_path) {
        Some(seqnum) => *seqnum,
        None => read_seqnum_from_storage(chain_state, &seqnum_access_path),
    }
}

fn read_balance_from_storage(
    chain_state: &dyn ChainState,
    balance_access_path: &AccessPath,
) -> u64 {
    read_u64_from_storage(chain_state, &balance_access_path)
}

fn read_seqnum_from_storage(chain_state: &dyn ChainState, seqnum_access_path: &AccessPath) -> u64 {
    read_u64_from_storage(chain_state, &seqnum_access_path)
}

fn read_u64_from_storage(chain_state: &dyn ChainState, access_path: &AccessPath) -> u64 {
    chain_state
        .get(&access_path)
        .expect("Failed to query storage.")
        .map_or(0, |bytes| decode_bytes(&bytes))
}

fn decode_bytes(bytes: &[u8]) -> u64 {
    let mut buf = [0; 8];
    buf.copy_from_slice(bytes);
    u64::from_le_bytes(buf)
}

fn balance_ap(account: AccountAddress) -> AccessPath {
    AccessPath::new(account, b"balance".to_vec())
}

fn seqnum_ap(account: AccountAddress) -> AccessPath {
    AccessPath::new(account, b"seqnum".to_vec())
}

fn gen_mint_writeset(sender: AccountAddress, balance: u64, seqnum: u64) -> WriteSet {
    let mut write_set = WriteSetMut::default();
    write_set.push((
        balance_ap(sender),
        WriteOp::Value(balance.to_le_bytes().to_vec()),
    ));
    write_set.push((
        seqnum_ap(sender),
        WriteOp::Value(seqnum.to_le_bytes().to_vec()),
    ));
    write_set.freeze().expect("mint writeset should be valid")
}

fn gen_payment_writeset(
    sender: AccountAddress,
    sender_balance: u64,
    sender_seqnum: u64,
    recipient: AccountAddress,
    recipient_balance: u64,
) -> WriteSet {
    let mut write_set = WriteSetMut::default();
    write_set.push((
        balance_ap(sender),
        WriteOp::Value(sender_balance.to_le_bytes().to_vec()),
    ));
    write_set.push((
        seqnum_ap(sender),
        WriteOp::Value(sender_seqnum.to_le_bytes().to_vec()),
    ));
    write_set.push((
        balance_ap(recipient),
        WriteOp::Value(recipient_balance.to_le_bytes().to_vec()),
    ));
    write_set
        .freeze()
        .expect("payment write set should be valid")
}

pub fn encode_mint_program(amount: u64) -> Script {
    let argument = TransactionArgument::U64(amount);
    Script::new(vec![], vec![argument])
}

pub fn encode_transfer_program(recipient: AccountAddress, amount: u64) -> Script {
    let argument1 = TransactionArgument::Address(recipient);
    let argument2 = TransactionArgument::U64(amount);
    Script::new(vec![], vec![argument1, argument2])
}

pub fn encode_mint_transaction(sender: AccountAddress, amount: u64) -> Transaction {
    encode_transaction(sender, encode_mint_program(amount))
}

pub fn encode_transfer_transaction(
    sender: AccountAddress,
    recipient: AccountAddress,
    amount: u64,
) -> Transaction {
    encode_transaction(sender, encode_transfer_program(recipient, amount))
}

fn encode_transaction(sender: AccountAddress, program: Script) -> Transaction {
    let raw_transaction =
        RawUserTransaction::new_script(sender, 0, program, 0, 0, std::time::Duration::from_secs(0));

    let (privkey, pubkey) = compat::generate_keypair(None);
    Transaction::UserTransaction(
        raw_transaction
            .sign(&privkey, pubkey)
            .expect("Failed to sign raw transaction.")
            .into_inner(),
    )
}

fn decode_transaction(txn: &SignedUserTransaction) -> MockTransaction {
    let sender = txn.sender();
    match txn.payload() {
        TransactionPayload::Script(script) => {
            assert!(script.code().is_empty(), "Code should be empty.");
            match script.args().len() {
                1 => match script.args()[0] {
                    TransactionArgument::U64(amount) => MockTransaction::Mint { sender, amount },
                    _ => unimplemented!(
                        "Only one integer argument is allowed for mint transactions."
                    ),
                },
                2 => match (&script.args()[0], &script.args()[1]) {
                    (TransactionArgument::Address(recipient), TransactionArgument::U64(amount)) => {
                        MockTransaction::Payment {
                            sender,
                            recipient: *recipient,
                            amount: *amount,
                        }
                    }
                    _ => unimplemented!(
                        "The first argument for payment transaction must be recipient address \
                         and the second argument must be amount."
                    ),
                },
                _ => unimplemented!("Transaction must have one or two arguments."),
            }
        }
        TransactionPayload::Module(_) => {
            unimplemented!("MockExecutor does not support Module transaction payload.")
        }
    }
}
