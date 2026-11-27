// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use clap::ValueEnum;
use starcoin_config::{
    temp_dir, BuiltinNetworkID, ChainNetwork, DataDirPath, RocksdbConfig, DEFAULT_CACHE_SIZE,
};
use starcoin_genesis::vm2::{build_genesis_transaction, execute_genesis_transaction};
use starcoin_metrics::metrics::VMMetrics;
use starcoin_metrics::Registry;
use starcoin_storage::cache_storage::CacheStorage;
use starcoin_storage::db_storage::DBStorage;
use starcoin_storage::storage::StorageInstance;
use starcoin_storage::Storage;
use starcoin_transaction_builder::vm2::{self as transaction_builder2, DEFAULT_MAX_GAS_AMOUNT};
use starcoin_vm2_cached_packages::starcoin_framework_sdk_builder::transfer_scripts_batch_peer_to_peer_v2;
use starcoin_vm2_crypto::keygen::KeyGen;
use starcoin_vm2_executor::block_executor;
use starcoin_vm2_statedb::ChainStateDB;
use starcoin_vm2_types::account_address;
use starcoin_vm2_types::account_address::AccountAddress;
use starcoin_vm2_vm_runtime::starcoin_vm::StarcoinVM;
use starcoin_vm2_vm_types::token::stc::stc_type_tag;
use starcoin_vm2_vm_types::transaction::authenticator::AccountPrivateKey;
use starcoin_vm2_vm_types::transaction::{
    RawUserTransaction, SignedUserTransaction, Transaction, TransactionPayload,
};
use std::cmp::min;
use std::sync::Arc;

const INIT_ACCOUNT_BALANCE: u64 = 40_000_000_000;
// Keep account creation batches small to avoid overwhelming caches.
const CREATE_BATCH_SIZE: usize = 500;
// Give generated transactions ample TTL to avoid expiration during generation/execution.
const TXN_TTL_SECS: u64 = 3600;

struct AccountData {
    private_key: AccountPrivateKey,
    address: AccountAddress,
    sequence_number: u64,
}

pub struct BenchmarkReport {
    concurrency_level: usize,
    txns: usize,
    exec_milliseconds: f64,
    tps: f64,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum VmStorage {
    Memory,
    Db,
}

struct TransactionGenerator {
    accounts: Vec<AccountData>,
    net: ChainNetwork,
}

struct TransactionExecutor<'test, S> {
    chain_state: &'test S,
}

impl TransactionGenerator {
    fn new(num_accounts: usize, net: ChainNetwork) -> Self {
        let mut accounts = Vec::with_capacity(num_accounts);
        let mut key_gen = KeyGen::from_os_rng();
        for _i in 0..num_accounts {
            let (private_key, public_key) = key_gen.generate_keypair();
            let address = account_address::from_public_key(&public_key);
            let account = AccountData {
                private_key: AccountPrivateKey::Single(private_key),
                address,
                sequence_number: 0,
            };
            accounts.push(account);
        }

        Self { accounts, net }
    }

    fn gen_create_account_transactions(&mut self) -> Vec<SignedUserTransaction> {
        // Generate creation txns in chunks to avoid exceeding association account sequence window.
        let mut all_txns = Vec::with_capacity(self.accounts.len());
        let chunk_size = CREATE_BATCH_SIZE; // stay within reasonable batch size
        for (chunk_idx, chunk) in self.accounts.chunks(chunk_size).enumerate() {
            for (offset, receiver) in chunk.iter().enumerate() {
                let seq = (chunk_idx * chunk_size + offset) as u64;
                let payload = transfer_scripts_batch_peer_to_peer_v2(
                    stc_type_tag(),
                    vec![receiver.address],
                    vec![INIT_ACCOUNT_BALANCE as u128],
                );

                let txn = transaction_builder2::create_signed_txn_with_association_account(
                    payload,
                    seq, // The first transaction from the association account should have sequence number 0
                    DEFAULT_MAX_GAS_AMOUNT,
                    1,
                    self.net.time_service().now_secs() + TXN_TTL_SECS,
                    self.net.chain_id().id().into(),
                    self.net.genesis_config2(),
                );

                all_txns.push(txn);
            }
        }

        all_txns
    }

    fn gen_transfer_transactions(&mut self, txns_num: usize) -> Vec<SignedUserTransaction> {
        let mut txns = Vec::with_capacity(txns_num);
        self.accounts.iter_mut().for_each(|account| {
            account.sequence_number = 0;
        });
        loop {
            // max accounts size is 200, it's ok to generate 100 txns that seperate from each other
            // testing machine didn't have so much cores
            for index in 0..self.accounts.len() / 2 {
                let sender_idx = 2 * index;
                let receiver_idx = 2 * index + 1;
                assert!(receiver_idx < self.accounts.len());
                let sender = &self.accounts[sender_idx];
                let receiver = &self.accounts[receiver_idx];
                let payload =
                    transaction_builder2::encode_transfer_script_function(receiver.address, 1);
                let txn = self.create_transaction_with_sender(
                    sender,
                    self.accounts[sender_idx].sequence_number,
                    payload,
                    self.net.time_service().now_secs() + TXN_TTL_SECS,
                    &self.net,
                );
                self.accounts[sender_idx].sequence_number += 1;
                txns.push(txn);
                if txns.len() >= txns_num {
                    return txns;
                }
            }
        }
    }

    fn create_transaction_with_sender(
        &self,
        sender: &AccountData,
        sequence_number: u64,
        payload: TransactionPayload,
        expiration_timestamp_secs: u64,
        net: &ChainNetwork,
    ) -> SignedUserTransaction {
        let raw_txn = RawUserTransaction::new_with_default_gas_token(
            sender.address,
            sequence_number,
            payload,
            DEFAULT_MAX_GAS_AMOUNT,
            1,
            expiration_timestamp_secs,
            net.chain_id().id().into(),
        );

        let signature = sender.private_key.sign(&raw_txn).unwrap();
        SignedUserTransaction::new(raw_txn, signature)
    }
}

impl<
        'test,
        S: starcoin_vm2_state_api::ChainStateReader + starcoin_vm2_state_api::ChainStateWriter + Sync,
    > TransactionExecutor<'test, S>
{
    fn new(chain_state: &'test S) -> Self {
        Self { chain_state }
    }

    fn run(&mut self, txns: Vec<SignedUserTransaction>, persist_result: bool) -> BenchmarkReport {
        let num_txns = txns.len();

        let registry = Registry::new();
        let vm_metrics = VMMetrics::register(&registry).ok();

        let user_txns: Vec<Transaction> =
            txns.into_iter().map(Transaction::UserTransaction).collect();

        if persist_result {
            let _ = block_executor::block_execute(
                self.chain_state,
                user_txns,
                u64::MAX,
                vm_metrics.clone(),
            )
            .expect("Execute txns fail.");
        } else {
            let _ = starcoin_vm2_executor::do_execute_block_transactions(
                self.chain_state,
                user_txns,
                Some(u64::MAX),
                vm_metrics.clone(),
            )
            .expect("Execute txns fail.");
        }

        self.chain_state.flush().expect("flush state should be ok");

        if let Some(ref metrics_reader) = vm_metrics {
            let execute_time_histogram = metrics_reader
                .vm_txn_exe_time
                .with_label_values(&["execute_transactions"]);
            let count = execute_time_histogram.get_sample_count();
            assert_eq!(count, 1);
            let execute_time_sum = execute_time_histogram.get_sample_sum();

            BenchmarkReport {
                concurrency_level: StarcoinVM::get_concurrency_level(),
                txns: num_txns,
                exec_milliseconds: execute_time_sum * 1000.0,
                tps: num_txns as f64 / execute_time_sum,
            }
        } else {
            BenchmarkReport {
                concurrency_level: StarcoinVM::get_concurrency_level(),
                txns: num_txns,
                exec_milliseconds: 0.0,
                tps: 0.0,
            }
        }
    }
}

pub struct BenchmarkManager {
    chain_state: ChainStateDB,
    net: ChainNetwork,
    _data_dir: Option<DataDirPath>,
}

impl BenchmarkManager {
    pub fn new(storage_mode: VmStorage) -> Self {
        // Default: in-memory cache storage; can switch to RocksDB via CLI.
        let (storage, data_dir) = match storage_mode {
            VmStorage::Memory => {
                // Default cache is small; enlarge to avoid eviction when creating many accounts.
                let cache_size = DEFAULT_CACHE_SIZE * 100;
                (
                    Arc::new(
                        Storage::new(StorageInstance::new_cache_instance_with_capacity(
                            cache_size,
                        ))
                        .expect("new cache storage should be ok"),
                    ),
                    None,
                )
            }
            VmStorage::Db => {
                let data_dir = temp_dir();
                let db = DBStorage::new(data_dir.path(), RocksdbConfig::default(), None)
                    .expect("new db storage should be ok");
                // Use a larger in-memory cache in front of RocksDB to avoid state node evictions
                // when creating many accounts.
                let cache = CacheStorage::new_with_capacity(
                    DEFAULT_CACHE_SIZE * 100,
                    /* metrics */ None,
                );
                (
                    Arc::new(
                        Storage::new(StorageInstance::new_cache_and_db_instance(cache, db))
                            .expect("new storage ok"),
                    ),
                    Some(data_dir),
                )
            }
        };
        let net: ChainNetwork = ChainNetwork::new_builtin(BuiltinNetworkID::Dev);
        let chain_state = ChainStateDB::new(storage.clone(), None);

        // Initialize genesis
        let genesis_txn = build_genesis_transaction(&net).unwrap();
        let _ =
            execute_genesis_transaction(&chain_state, Transaction::UserTransaction(genesis_txn))
                .unwrap();
        Self {
            chain_state,
            net,
            _data_dir: data_dir,
        }
    }

    pub fn run(
        &mut self,
        serialize_bench_txns: &[usize],
        parallel_bench_txns: &[usize],
        account_override: Option<usize>,
        concurrency_override: Option<usize>,
    ) -> Vec<BenchmarkReport> {
        let mut reports = Vec::new();

        // generate account
        let max_txns_once = serialize_bench_txns
            .iter()
            .chain(parallel_bench_txns.iter())
            .max()
            .copied()
            .unwrap_or(0);
        // Default accounts: 2x max txns (cap 20000) to reduce conflicts; CLI can override.
        let mut account_num = account_override.unwrap_or_else(|| min(max_txns_once * 2, 20000));
        if account_num == 0 {
            account_num = 1;
        }

        let mut generator = TransactionGenerator::new(account_num, self.net.clone());
        let mut executor = TransactionExecutor::new(&self.chain_state);
        let create_txns = generator.gen_create_account_transactions();
        // Persist creation txns in batches to avoid huge single-block state and eviction issues.
        for chunk in create_txns.chunks(CREATE_BATCH_SIZE) {
            let _ = executor.run(chunk.to_vec(), true);
        }

        // run serialize txns
        for txns_num in serialize_bench_txns.iter() {
            let txns = generator.gen_transfer_transactions(*txns_num);
            reports.push(executor.run(txns, false));
        }

        // this variable could only be set once, default is serialize, so we run serialize first.
        let desired_parallel = concurrency_override
            .unwrap_or_else(num_cpus::get)
            .clamp(1, num_cpus::get());
        StarcoinVM::set_concurrency_level(desired_parallel);
        assert_eq!(StarcoinVM::get_concurrency_level(), desired_parallel);

        // run parallel txns
        for txns_num in parallel_bench_txns.iter() {
            let txns = generator.gen_transfer_transactions(*txns_num);
            reports.push(executor.run(txns, false));
        }

        reports
    }

    pub fn pretty_print_reports(&mut self, reports: &[BenchmarkReport]) {
        println!("┌─────────────┬──────────┬─────────────┬─────────────┐");
        println!("│ Concurrency │   Txns   │  Exec(ms)   │     TPS     │");
        println!("├─────────────┼──────────┼─────────────┼─────────────┤");

        for report in reports {
            println!(
                "│ {:^11} │ {:^8} │ {:^11.2} │ {:^11.2} │",
                report.concurrency_level, report.txns, report.exec_milliseconds, report.tps
            );
        }

        println!("└─────────────┴──────────┴─────────────┴─────────────┘");
    }
}

impl Default for BenchmarkManager {
    fn default() -> Self {
        Self::new(VmStorage::Memory)
    }
}
