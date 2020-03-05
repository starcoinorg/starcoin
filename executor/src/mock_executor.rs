// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::TransactionExecutor;
use anyhow::{Error, Result};
use config::VMConfig;
use crypto::{ed25519::compat, ed25519::*, hash::CryptoHash, traits::SigningKey, HashValue};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use traits::{ChainState, ChainStateReader, ChainStateWriter};
use types::{
    access_path::AccessPath,
    account_address::{AccountAddress, ADDRESS_LENGTH},
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
use vm_runtime::{mock_vm::MockVM, account::AccountData};



const MOCK_GAS_AMOUNT: u64 = 140_000;
const MOCK_GAS_PRICE: u64 = 1;

pub struct MockChainState {
    //    state_tree: SparseMerkleTree,
}

impl MockChainState {
    // create empty chain state
    pub fn new() -> Self {
        MockChainState {
//            state_tree: empty_tree(),
        }
    }
    /// Commit and calculate new state root
    pub fn commit(&self) -> Result<HashValue> {
        unimplemented!()
    }

    /// flush data to db.
    pub fn flush(&self) -> Result<()> {
        unimplemented!()
    }
}

impl ChainState for MockChainState {}

impl ChainStateReader for MockChainState {
    fn get(&self, access_path: &AccessPath) -> Result<Option<Vec<u8>>, Error> {
        Ok(None)
    }

    fn get_code(&self, module_id: &ModuleId) -> Result<Option<Vec<u8>>> {
        unimplemented!()
    }

    fn get_account_state(&self, address: &AccountAddress) -> Result<Option<AccountState>> {
        Ok(None)
    }

    fn is_genesis(&self) -> bool {
        unimplemented!()
    }

    fn state_root(&self) -> HashValue {
        unimplemented!()
    }
}

impl ChainStateWriter for MockChainState {
    fn set(&self, access_path: &AccessPath, value: Vec<u8>) -> Result<()> {
        Ok(())
    }

    fn delete(&self, access_path: &AccessPath) -> Result<()> {
        unimplemented!()
    }

    fn delete_at(&self, account_state: &AccountState, struct_tag: &StructTag) -> Result<()> {
        unimplemented!()
    }

    fn set_code(&self, module_id: &ModuleId, code: Vec<u8>) -> Result<(), Error> {
        unimplemented!()
    }

    fn create_account(&self, account_address: AccountAddress) -> Result<(), Error> {
        unimplemented!()
    }
}

pub struct MockExecutor {
    config: VMConfig,
}

impl MockExecutor {
    /// Creates an executor in which no genesis state has been applied yet.
    pub fn new() -> Self {
        MockExecutor {
            config: VMConfig::default(),
        }
    }
    pub fn add_account_data(&mut self, account_data: &AccountData, chain_state: &dyn ChainState) {
        let mut vm = MockVM::new(&self.config);
        vm.add_account_data(account_data, chain_state)
    }
}

impl TransactionExecutor for MockExecutor {
    fn execute_transaction(
        config: &VMConfig,
        chain_state: &dyn ChainState,
        txn: Transaction,
    ) -> Result<TransactionOutput> {
        let mut vm = MockVM::new(config);
        let output = vm.execute_transaction(chain_state, txn);
        Ok(output)
    }

    fn validate_transaction(
        config: &VMConfig,
        chain_state: &dyn ChainState,
        txn: SignedUserTransaction,
    ) -> Option<VMStatus> {
        None
    }
}


pub fn get_signed_txn(
    sender: AccountAddress,
    sequence_number: u64,
    private_key: &Ed25519PrivateKey,
    public_key: Ed25519PublicKey,
    script: Script,
) -> SignedUserTransaction {
    let expiration_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + 10; // 10 seconds from now.
    let raw_txn = RawUserTransaction::new_script(
        sender,
        sequence_number,
        script,
        MOCK_GAS_AMOUNT,
        MOCK_GAS_PRICE,
        Duration::from_secs(expiration_time),
    );

    let signature = private_key.sign_message(&raw_txn.crypto_hash());

    SignedUserTransaction::new(raw_txn, public_key, signature)
}
