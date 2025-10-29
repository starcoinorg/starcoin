// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_config::{BuiltinNetworkID, ChainNetwork};
use starcoin_genesis::vm2::{build_genesis_transaction, execute_genesis_transaction};
use starcoin_metrics::metrics::VMMetrics;
use starcoin_metrics::Registry;
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
        self.net.time_service().sleep(1000);
        let mut txns = vec![];
        for (sequence_number, receiver) in self.accounts.iter().enumerate() {
            let payload = transfer_scripts_batch_peer_to_peer_v2(
                stc_type_tag(),
                vec![receiver.address],
                vec![INIT_ACCOUNT_BALANCE as u128],
            );

            let txn = transaction_builder2::create_signed_txn_with_association_account(
                payload,
                sequence_number as u64, // The first transaction from the association account should have sequence number 0
                DEFAULT_MAX_GAS_AMOUNT,
                1,
                self.net.time_service().now_secs() + sequence_number as u64,
                self.net.chain_id().id().into(),
                self.net.genesis_config2(),
            );

            txns.push(txn);
        }

        txns
    }

    fn gen_transfer_transactions(&mut self, txns_num: usize) -> Vec<SignedUserTransaction> {
        self.net.time_service().sleep(1000);
        let mut txns = Vec::with_capacity(txns_num);
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
                    self.net.time_service().now_secs()
                        + self.accounts[sender_idx].sequence_number
                        + 1,
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
}

impl BenchmarkManager {
    pub fn new() -> Self {
        let storage = Arc::new(
            Storage::new(StorageInstance::new_cache_instance()).expect("new storage should be ok"),
        );
        let net: ChainNetwork = ChainNetwork::new_builtin(BuiltinNetworkID::Dev);
        let chain_state = ChainStateDB::new(storage.clone(), None);

        // Initialize genesis
        let genesis_txn = build_genesis_transaction(&net).unwrap();
        let _ =
            execute_genesis_transaction(&chain_state, Transaction::UserTransaction(genesis_txn))
                .unwrap();
        Self { chain_state, net }
    }

    pub fn run(
        &mut self,
        serialize_bench_txns: &[usize],
        parallel_bench_txns: &[usize],
    ) -> Vec<BenchmarkReport> {
        let mut reports = Vec::new();

        // generate account
        let max_txns_once = serialize_bench_txns
            .iter()
            .chain(parallel_bench_txns.iter())
            .max()
            .copied()
            .unwrap_or(0);
        // 200 account is enough to avoid conflict in parallel execution.
        let account_num = min(max_txns_once * 2, 200);

        let mut generator = TransactionGenerator::new(account_num, self.net.clone());
        let txns = generator.gen_create_account_transactions();
        let mut executor = TransactionExecutor::new(&self.chain_state);
        let _ = executor.run(txns, true);

        // do not persist the execution result to storage to save benchmark time
        let do_not_persist_result = (serialize_bench_txns.len() + parallel_bench_txns.len()) <= 1;

        // run serialize txns
        for txns_num in serialize_bench_txns.iter() {
            let txns = generator.gen_transfer_transactions(*txns_num);
            reports.push(executor.run(txns, do_not_persist_result));
        }

        // this variable could only be set once, default is serialize, so we run serialize first.
        StarcoinVM::set_concurrency_level_once(num_cpus::get());
        assert_eq!(StarcoinVM::get_concurrency_level(), num_cpus::get());

        // run parallel txns
        for txns_num in parallel_bench_txns.iter() {
            let txns = generator.gen_transfer_transactions(*txns_num);
            reports.push(executor.run(txns, do_not_persist_result));
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
        Self::new()
    }
}
