// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::TransactionExecutor;
use anyhow::{Error, Result};
use compiler::compile::StarcoinCompiler;
use config::VMConfig;
use crypto::{ed25519::compat, ed25519::*, hash::CryptoHash, traits::SigningKey, HashValue};
use once_cell::sync::Lazy;
use state_tree::mock::MockStateNodeStore;
use statedb::ChainStateDB;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use traits::{ChainState, ChainStateReader, ChainStateWriter};
use types::{
    access_path::AccessPath,
    account_address::{AccountAddress, ADDRESS_LENGTH},
    account_config::{account_struct_tag, association_address, AccountResource},
    account_state::AccountState,
    contract_event::ContractEvent,
    language_storage::{ModuleId, StructTag, TypeTag},
    state_set::ChainStateSet,
    transaction::{
        RawUserTransaction, Script, SignedUserTransaction, Transaction, TransactionArgument,
        TransactionOutput, TransactionPayload, TransactionStatus,
    },
    vm_error::{StatusCode, VMStatus},
};
use vm_runtime::mock_vm::{encode_mint_transaction, encode_transfer_transaction, MockVM};

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

    fn dump(&self) -> Result<ChainStateSet> {
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

    fn apply(&self, state_set: ChainStateSet) -> Result<(), Error> {
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
}

impl TransactionExecutor for MockExecutor {
    fn init_genesis(_config: &VMConfig) -> Result<(HashValue, ChainStateSet)> {
        let chain_state = ChainStateDB::new(Arc::new(MockStateNodeStore::new()), None);
        chain_state.create_account(AccountAddress::default())?;
        chain_state.create_account(association_address())?;
        chain_state.commit();
        Ok((chain_state.state_root(), chain_state.dump()?))
    }

    fn execute_transaction(
        config: &VMConfig,
        chain_state: &dyn ChainState,
        txn: Transaction,
    ) -> Result<TransactionOutput> {
        let mut vm = MockVM::new(config);
        let output = vm.execute_transaction(chain_state, txn);
        output
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

pub fn mock_txn() -> Transaction {
    let empty_script = StarcoinCompiler::compile_script("main() {return;}");
    Transaction::UserTransaction(SignedUserTransaction::mock_from(empty_script))
}

pub fn mock_mint_txn(chain_state: &dyn ChainState) -> Transaction {
    let account_address = AccountAddress::random();
    chain_state.create_account(account_address);
    encode_mint_transaction(account_address, 100)
}

pub fn mock_transfer_txn(account_address: AccountAddress, amount: u64) -> Transaction {
    encode_mint_transaction(account_address, amount)
}
