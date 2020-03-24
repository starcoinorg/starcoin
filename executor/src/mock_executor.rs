// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::TransactionExecutor;
use anyhow::{Error, Result};

use config::VMConfig;
use crypto::{ed25519::*, hash::CryptoHash, traits::SigningKey, HashValue};

use state_tree::mock::MockStateNodeStore;
use statedb::ChainStateDB;

use std::convert::TryInto;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use stdlib::transaction_scripts::EMPTY_TXN;
use traits::{ChainState, ChainStateReader, ChainStateWriter};
use types::{
    access_path::AccessPath,
    account_address::AccountAddress,
    account_config::{association_address, AccountResource},
    account_state::AccountState,
    state_set::ChainStateSet,
    transaction::{
        RawUserTransaction, Script, SignedUserTransaction, Transaction, TransactionOutput,
    },
    vm_error::VMStatus,
};
use vm_runtime::mock_vm::{
    encode_transfer_program, encode_transfer_transaction, mock_transaction_with_seq_number, MockVM,
};

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
    fn get(&self, _access_path: &AccessPath) -> Result<Option<Vec<u8>>, Error> {
        Ok(None)
    }

    fn get_account_state(&self, _address: &AccountAddress) -> Result<Option<AccountState>> {
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
    fn set(&self, _access_path: &AccessPath, _value: Vec<u8>) -> Result<()> {
        Ok(())
    }

    fn remove(&self, _access_path: &AccessPath) -> Result<()> {
        unimplemented!()
    }

    fn create_account(&self, _account_address: AccountAddress) -> Result<(), Error> {
        unimplemented!()
    }

    fn apply(&self, _state_set: ChainStateSet) -> Result<(), Error> {
        unimplemented!()
    }
    fn commit(&self) -> Result<HashValue> {
        unimplemented!()
    }

    fn flush(&self) -> Result<()> {
        unimplemented!()
    }
}

#[derive(Clone)]
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

    fn mint_for(chain_state: &dyn ChainState, account: AccountAddress, amount: u64) -> Result<()> {
        let access_path = AccessPath::new_for_account(account);
        let account_resource: AccountResource = chain_state
            .get(&access_path)
            .and_then(|blob| match blob {
                Some(blob) => Ok(blob),
                None => {
                    chain_state.create_account(account)?;
                    Ok(chain_state
                        .get(&access_path)?
                        .expect("account resource must exist."))
                }
            })?
            .try_into()?;
        let new_account_resource = AccountResource::new(
            account_resource.balance() + amount,
            account_resource.sequence_number(),
            account_resource.authentication_key().clone(),
        );
        chain_state.set(&access_path, new_account_resource.try_into()?);
        Ok(())
    }
}

impl TransactionExecutor for MockExecutor {
    fn init_genesis(_config: &VMConfig) -> Result<(HashValue, ChainStateSet)> {
        let chain_state = ChainStateDB::new(Arc::new(MockStateNodeStore::new()), None);
        Self::mint_for(&chain_state, AccountAddress::default(), 10_0000_0000_0000)?;
        chain_state.create_account(association_address())?;
        chain_state.commit()?;
        chain_state.flush()?;
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
        let mut vm = MockVM::new(config);
        vm.verify_transaction(chain_state, txn)
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
    let empty_script = EMPTY_TXN.clone();
    Transaction::UserTransaction(SignedUserTransaction::mock_from(empty_script))
}

pub fn mock_mint_txn(to: AccountAddress, amount: u64) -> Transaction {
    let from = AccountAddress::default();
    encode_transfer_transaction(from, to, amount)
}

pub fn mock_transfer_txn(from: AccountAddress, to: AccountAddress, amount: u64) -> Transaction {
    encode_transfer_transaction(from, to, amount)
}

pub fn mock_transfer_txn_with_seq_number(
    sender_sequence_number: u64,
    from: AccountAddress,
    to: AccountAddress,
    amount: u64,
) -> Transaction {
    let script = encode_transfer_program(to, amount);
    mock_transaction_with_seq_number(from, sender_sequence_number, script)
}
