// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::TransactionExecutor;
use anyhow::Result;
use config::VMConfig;
use crypto::{ed25519::*, hash::CryptoHash, traits::SigningKey, HashValue};
use state_tree::mock::MockStateNodeStore;
use statedb::ChainStateDB;
use starcoin_state_api::{ChainState, ChainStateReader, ChainStateWriter};
use std::convert::TryInto;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use stdlib::transaction_scripts::EMPTY_TXN;
use types::{
    access_path::AccessPath,
    account_address::AccountAddress,
    account_config::{association_address, AccountResource},
    state_set::ChainStateSet,
    transaction::{
        RawUserTransaction, Script, SignedUserTransaction, Transaction, TransactionOutput,
    },
    vm_error::VMStatus,
};
use vm_runtime::mock_vm::{
    encode_transfer_program, encode_transfer_transaction, mock_raw_transfer_txn,
    mock_transaction_with_seq_number, MockVM,
};

const MOCK_GAS_AMOUNT: u64 = 140_000;
const MOCK_GAS_PRICE: u64 = 1;

#[derive(Clone)]
pub struct MockExecutor {}

impl MockExecutor {
    /// Creates an executor in which no genesis state has been applied yet.
    pub fn new() -> Self {
        MockExecutor {}
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
            account_resource.authentication_key().to_vec(),
        );
        chain_state.set(&access_path, new_account_resource.try_into()?)?;
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

    fn build_mint_txn(
        addr: AccountAddress,
        _auth_key_prefix: Vec<u8>,
        _seq_num: u64,
        amount: u64,
    ) -> Transaction {
        let from = AccountAddress::default();
        encode_transfer_transaction(from, addr, amount)
    }

    fn build_transfer_txn(
        sender: AccountAddress,
        _sender_auth_key_prefix: Vec<u8>,
        receiver: AccountAddress,
        _receiver_auth_key_prefix: Vec<u8>,
        seq_num: u64,
        amount: u64,
    ) -> RawUserTransaction {
        mock_raw_transfer_txn(sender, receiver, amount, seq_num)
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
