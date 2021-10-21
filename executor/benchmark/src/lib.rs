// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crypto::keygen::KeyGen;
use crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey},
    HashValue, PrivateKey, Uniform,
};
use executor::{encode_create_account_script_function, encode_transfer_script_function};
use logger::prelude::*;
use rand::{rngs::StdRng, SeedableRng};
use starcoin_config::ChainNetwork;
use starcoin_genesis::Genesis;
use starcoin_state_api::ChainState;
use starcoin_vm_types::genesis_config::StdlibVersion;
use starcoin_vm_types::token::stc;
use starcoin_vm_types::transaction::authenticator::AuthenticationKey;
use starcoin_vm_types::transaction::ScriptFunction;
use statedb::ChainStateDB;
use std::sync::mpsc;
use std::sync::Arc;
use storage::storage::StorageInstance;
use storage::Storage;
use types::{
    account_address,
    account_address::AccountAddress,
    block_metadata::BlockMetadata,
    transaction::{Transaction, TransactionPayload},
};

struct AccountData {
    public_key: Ed25519PublicKey,
    address: AccountAddress,
}

impl AccountData {
    pub fn public_key(&self) -> &Ed25519PublicKey {
        &self.public_key
    }
    pub fn random() -> Self {
        let mut key_gen = KeyGen::from_os_rng();
        let (_private_key, public_key) = key_gen.generate_keypair();
        let address = account_address::from_public_key(&public_key);
        AccountData {
            public_key,
            address,
        }
    }
}

struct TransactionGenerator {
    /// The current state of the accounts. The main purpose is to keep track of the sequence number
    /// so generated transactions are guaranteed to be successfully executed.
    accounts: Vec<AccountData>,

    /// For deterministic transaction generation.
    rng: StdRng,

    /// Each generated block of transactions are sent to this channel. Using `SyncSender` to make
    /// sure if execution is slow to consume the transactions, we do not run out of memory.
    block_sender: Option<mpsc::SyncSender<Vec<Transaction>>>,

    sequence: u64,

    net: ChainNetwork,

    block_number: u64,
}

impl TransactionGenerator {
    fn new(
        num_accounts: usize,
        block_sender: mpsc::SyncSender<Vec<Transaction>>,
        net: ChainNetwork,
    ) -> Self {
        let seed = [1u8; 32];
        let mut rng = StdRng::from_seed(seed);

        let mut accounts = Vec::with_capacity(num_accounts);
        for _i in 0..num_accounts {
            let private_key = Ed25519PrivateKey::generate(&mut rng);
            let public_key = private_key.public_key();
            let address = account_address::from_public_key(&public_key);
            let account = AccountData {
                public_key,
                address,
            };
            accounts.push(account);
        }

        Self {
            accounts,
            rng,
            block_sender: Some(block_sender),
            sequence: 0,
            net,
            block_number: 1,
        }
    }

    fn run(&mut self, init_account_balance: u64, block_size: usize, num_transfer_blocks: usize) {
        self.gen_create_account_transactions(init_account_balance, block_size);
        self.gen_transfer_transactions(block_size, num_transfer_blocks);
    }

    /// Generates transactions that allocate `init_account_balance` to every account.
    fn gen_create_account_transactions(&mut self, init_account_balance: u64, block_size: usize) {
        for (_i, block) in self.accounts.chunks(block_size).enumerate() {
            self.net.time_service().sleep(1000);

            let mut transactions = Vec::with_capacity(block_size + 1);
            let minter_account = AccountData::random();
            let block_meta = BlockMetadata::new(
                HashValue::random(),
                self.net.time_service().now_millis(),
                minter_account.address,
                Some(AuthenticationKey::ed25519(&minter_account.public_key)),
                0,
                self.block_number,
                self.net.chain_id(),
                0,
            );
            self.block_number += 1;
            transactions.push(Transaction::BlockMetadata(block_meta));

            for (j, account) in block.iter().enumerate() {
                let txn = create_transaction(
                    self.sequence,
                    encode_create_account_script_function(
                        StdlibVersion::Latest,
                        stc::stc_type_tag(),
                        &account.address,
                        AuthenticationKey::ed25519(account.public_key()),
                        init_account_balance as u128,
                    ),
                    self.net.time_service().now_secs() + j as u64 + 1,
                    &self.net,
                );
                transactions.push(txn);
                self.sequence += 1;
            }

            self.block_sender
                .as_ref()
                .unwrap()
                .send(transactions)
                .unwrap();
        }
    }

    /// Generates transactions for random pairs of accounts.
    fn gen_transfer_transactions(&mut self, block_size: usize, num_blocks: usize) {
        for _i in 0..num_blocks {
            self.net.time_service().sleep(1000);
            let mut transactions = Vec::with_capacity(block_size + 1);
            let minter_account = AccountData::random();
            let block_meta = BlockMetadata::new(
                HashValue::random(),
                self.net.time_service().now_millis(),
                minter_account.address,
                Some(AuthenticationKey::ed25519(&minter_account.public_key)),
                0,
                self.block_number,
                self.net.chain_id(),
                0,
            );
            self.block_number += 1;
            transactions.push(Transaction::BlockMetadata(block_meta));

            for j in 0..block_size {
                let indices = rand::seq::index::sample(&mut self.rng, self.accounts.len(), 1);
                //                let sender_idx = indices.index(0);
                let receiver_idx = indices.index(0);

                //                let sender = &self.accounts[sender_idx];
                let receiver = &self.accounts[receiver_idx];
                let txn = create_transaction(
                    self.sequence,
                    encode_transfer_script_function(receiver.address, 1 /* amount */),
                    self.net.time_service().now_secs() + j as u64 + 1,
                    &self.net,
                );
                transactions.push(txn);

                self.sequence += 1;
            }

            self.block_sender
                .as_ref()
                .unwrap()
                .send(transactions)
                .unwrap();
        }
    }

    /// Drops the sender to notify the receiving end of the channel.
    fn drop_sender(&mut self) {
        self.block_sender.take().unwrap();
    }
}

struct TxnExecutor<'test> {
    chain_state: &'test dyn ChainState,
    block_receiver: mpsc::Receiver<Vec<Transaction>>,
}

impl<'test> TxnExecutor<'test> {
    fn new(
        chain_state: &'test dyn ChainState,
        block_receiver: mpsc::Receiver<Vec<Transaction>>,
    ) -> Self {
        Self {
            chain_state,
            block_receiver,
        }
    }

    fn run(&mut self) {
        let mut version = 0;
        while let Ok(transactions) = self.block_receiver.recv() {
            let execute_start = std::time::Instant::now();
            let num_txns = transactions.len();
            version += num_txns as u64;

            let _ = executor::block_execute(self.chain_state, transactions, u64::MAX, None)
                .expect("Execute transactions fail.");
            self.chain_state.flush().expect("flush state should be ok");

            let execute_time = std::time::Instant::now().duration_since(execute_start);
            let commit_start = std::time::Instant::now();

            let commit_time = std::time::Instant::now().duration_since(commit_start);
            let total_time = execute_time + commit_time;

            info!(
                "Version: {}. execute time: {} ms. commit time: {} ms. TPS: {}.",
                version,
                execute_time.as_millis(),
                commit_time.as_millis(),
                num_txns as u128 * 1_000_000_000 / total_time.as_nanos(),
            );
        }
    }
}

/// Runs the benchmark with given parameters.
pub fn run_benchmark(
    num_accounts: usize,
    init_account_balance: u64,
    block_size: usize,
    num_transfer_blocks: usize,
) {
    let storage = Arc::new(Storage::new(StorageInstance::new_cache_instance()).unwrap());

    let chain_state = ChainStateDB::new(storage, None);
    let net = ChainNetwork::new_test();
    let genesis_txn = Genesis::build_genesis_transaction(&net).unwrap();
    let _txn_info = Genesis::execute_genesis_txn(&chain_state, genesis_txn).unwrap();

    let (block_sender, block_receiver) = mpsc::sync_channel(50 /* bound */);

    // Spawn two threads to run transaction generator and executor separately.
    let gen_thread = std::thread::Builder::new()
        .name("txn_generator".to_string())
        .spawn(move || {
            let mut generator = TransactionGenerator::new(num_accounts, block_sender, net);
            generator.run(init_account_balance, block_size, num_transfer_blocks);
            generator
        })
        .expect("Failed to spawn transaction generator thread.");
    let exe_thread = std::thread::Builder::new()
        .name("txn_executor".to_string())
        .spawn(move || {
            let mut exe = TxnExecutor::new(&chain_state, block_receiver);
            exe.run();
        })
        .expect("Failed to spawn transaction executor thread.");

    // Wait for generator to finish and get back the generator.
    let mut generator = gen_thread.join().unwrap();
    // Drop the sender so the executor thread can eventually exit.
    generator.drop_sender();
    // Wait until all transactions are committed.
    exe_thread.join().unwrap();
}

fn create_transaction(
    sequence_number: u64,
    program: ScriptFunction,
    expiration_timestamp_secs: u64,
    net: &ChainNetwork,
) -> Transaction {
    let signed_txn = executor::create_signed_txn_with_association_account(
        TransactionPayload::ScriptFunction(program),
        sequence_number,
        40_000_000,
        1,
        expiration_timestamp_secs,
        net,
    );
    Transaction::UserTransaction(signed_txn)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_benchmark() {
        super::run_benchmark(
            25,        /* num_accounts */
            1_000_000, /* init_account_balance */
            5,         /* block_size */
            5,         /* num_transfer_blocks */
        );
    }
}
