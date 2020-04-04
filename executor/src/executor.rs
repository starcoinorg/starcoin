// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::TransactionExecutor;
use anyhow::Result;
use crypto::HashValue;
use starcoin_config::{ChainConfig, VMConfig};
use starcoin_state_api::{ChainState, ChainStateReader, ChainStateWriter};
use statedb::ChainStateDB;
use std::sync::Arc;
use storage::{cache_storage::CacheStorage, storage::StorageInstance, Storage};
use types::{
    account_address::AccountAddress,
    state_set::ChainStateSet,
    transaction::{RawUserTransaction, SignedUserTransaction, Transaction, TransactionOutput},
    vm_error::VMStatus,
};
use vm_runtime::genesis::generate_genesis_state_set;
use vm_runtime::{
    account::Account,
    common_transactions::{
        create_account_txn_sent_as_association, peer_to_peer_txn_sent_as_association,
        raw_peer_to_peer_txn,
    },
    starcoin_vm::StarcoinVM,
};

#[derive(Clone)]
pub struct Executor {}

impl Executor {
    /// Creates an executor in which no genesis state has been applied yet.
    pub fn new() -> Self {
        Executor {}
    }
}

impl TransactionExecutor for Executor {
    fn init_genesis(chain_config: &ChainConfig) -> Result<(HashValue, ChainStateSet)> {
        let storage = Arc::new(Storage::new(StorageInstance::new_cache_instance(
            CacheStorage::new(),
        ))?);
        let chain_state_db = ChainStateDB::new(storage, None);

        generate_genesis_state_set(&chain_config, &chain_state_db)?;
        chain_state_db.commit()?;
        chain_state_db.flush()?;

        let dump = chain_state_db.dump()?;
        Ok((chain_state_db.state_root(), dump))
    }

    fn execute_transaction(
        config: &VMConfig,
        chain_state: &dyn ChainState,
        txn: Transaction,
    ) -> Result<TransactionOutput> {
        let mut vm = StarcoinVM::new(config);
        let output = vm.execute_transaction(chain_state, txn);
        Ok(output)
    }

    fn validate_transaction(
        config: &VMConfig,
        chain_state: &dyn ChainState,
        txn: SignedUserTransaction,
    ) -> Option<VMStatus> {
        let mut vm = StarcoinVM::new(config);
        vm.verify_transaction(chain_state, txn)
    }

    fn build_mint_txn(
        addr: AccountAddress,
        auth_key_prefix: Vec<u8>,
        seq_num: u64,
        amount: u64,
    ) -> Transaction {
        Transaction::UserTransaction(peer_to_peer_txn_sent_as_association(
            addr,
            auth_key_prefix,
            seq_num,
            amount,
        ))
    }

    fn build_transfer_txn(
        sender: AccountAddress,
        sender_auth_key_prefix: Vec<u8>,
        receiver: AccountAddress,
        receiver_auth_key_prefix: Vec<u8>,
        seq_num: u64,
        amount: u64,
    ) -> RawUserTransaction {
        raw_peer_to_peer_txn(
            sender,
            sender_auth_key_prefix,
            receiver,
            receiver_auth_key_prefix,
            amount,
            seq_num,
        )
    }
}

pub fn mock_create_account_txn() -> Transaction {
    let account1 = Account::new();
    Transaction::UserTransaction(create_account_txn_sent_as_association(
        &account1, 1, // fix me
        1_000,
    ))
}
