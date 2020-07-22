// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey},
    HashValue, PrivateKey, Uniform,
};
use executor::{encode_create_account_script, encode_transfer_script};
use logger::prelude::*;
use rand::{rngs::StdRng, SeedableRng};
use starcoin_config::ChainNetwork;
use starcoin_genesis::Genesis;
use starcoin_state_api::ChainState;
use statedb::ChainStateDB;
use std::sync::mpsc;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use storage::cache_storage::CacheStorage;
use storage::storage::StorageInstance;
use storage::Storage;
use types::{
    account_address,
    account_address::AccountAddress,
    block_metadata::BlockMetadata,
    transaction::{authenticator::AuthenticationKey, Script, Transaction, TransactionPayload},
};

struct AccountData {
    public_key: Ed25519PublicKey,
    address: AccountAddress,
}

impl AccountData {
    pub fn auth_key_prefix(&self) -> Vec<u8> {
        AuthenticationKey::ed25519(&self.public_key)
            .prefix()
            .to_vec()
    }
    pub fn random() -> Self {
        let seed = [1u8; 32];
        let mut rng = StdRng::from_seed(seed);
        let private_key = Ed25519PrivateKey::generate(&mut rng);
        let public_key = private_key.public_key();
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
}

impl TransactionGenerator {
    fn new(num_accounts: usize, block_sender: mpsc::SyncSender<Vec<Transaction>>) -> Self {
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
        }
    }

    fn run(&mut self, init_account_balance: u64, block_size: usize, num_transfer_blocks: usize) {
        self.gen_mint_transactions(init_account_balance, block_size);
        self.gen_transfer_transactions(block_size, num_transfer_blocks);
    }

    /// Generates transactions that allocate `init_account_balance` to every account.
    fn gen_mint_transactions(&mut self, init_account_balance: u64, block_size: usize) {
        for (_i, block) in self.accounts.chunks(block_size).enumerate() {
            let mut transactions = Vec::with_capacity(block_size);
            for (_j, account) in block.iter().enumerate() {
                let txn = create_transaction(
                    self.sequence,
                    encode_create_account_script(
                        &account.address,
                        account.auth_key_prefix(),
                        init_account_balance,
                    ),
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
            let mut transactions = Vec::with_capacity(block_size);
            for _j in 0..block_size {
                let indices = rand::seq::index::sample(&mut self.rng, self.accounts.len(), 2);
                //                let sender_idx = indices.index(0);
                let receiver_idx = indices.index(1);

                //                let sender = &self.accounts[sender_idx];
                let receiver = &self.accounts[receiver_idx];
                let txn = create_transaction(
                    self.sequence,
                    encode_transfer_script(
                        &receiver.address,
                        receiver.auth_key_prefix(),
                        1, /* amount */
                    ),
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
        let miner_account = AccountData::random();
        while let Ok(mut transactions) = self.block_receiver.recv() {
            let execute_start = std::time::Instant::now();
            {
                let block_meta = BlockMetadata::new(
                    HashValue::random(),
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .expect("Clock may have gone backwards")
                        .as_secs(),
                    AccountAddress::random(),
                    Some(miner_account.auth_key_prefix()),
                    0,
                );

                transactions.insert(0, Transaction::BlockMetadata(block_meta));
            };

            let num_txns = transactions.len();
            version += num_txns as u64;

            let _ = executor::block_execute(self.chain_state, transactions, u64::MAX)
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
    let cache_storage = CacheStorage::new();
    let storage =
        Arc::new(Storage::new(StorageInstance::new_cache_instance(cache_storage)).unwrap());

    let chain_state = ChainStateDB::new(storage, None);

    let genesis_txn = Genesis::build_genesis_transaction(ChainNetwork::Dev).unwrap();
    let _txn_info = Genesis::execute_genesis_txn(&chain_state, genesis_txn).unwrap();

    let (block_sender, block_receiver) = mpsc::sync_channel(50 /* bound */);

    // Spawn two threads to run transaction generator and executor separately.
    let gen_thread = std::thread::Builder::new()
        .name("txn_generator".to_string())
        .spawn(move || {
            let mut generator = TransactionGenerator::new(num_accounts, block_sender);
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

fn create_transaction(sequence_number: u64, program: Script) -> Transaction {
    let signed_txn = executor::create_signed_txn_with_association_account(
        TransactionPayload::Script(program),
        sequence_number,
        400_000,
        1,
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
