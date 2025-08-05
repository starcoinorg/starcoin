// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use rand::{rngs::StdRng, SeedableRng};
use starcoin_config::ChainNetwork;
use starcoin_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey},
    PrivateKey, Uniform,
};
use starcoin_genesis::Genesis;
use starcoin_state_api::{ChainStateReader, ChainStateWriter};
use starcoin_statedb::ChainStateDB;
use starcoin_storage::storage::StorageInstance;
use starcoin_storage::Storage;
use starcoin_transaction_builder::{
    encode_create_account_script_function, encode_transfer_script_function,
};
use starcoin_types::transaction::RawUserTransaction;
use starcoin_types::{
    account_address,
    account_address::AccountAddress,
    transaction::{Transaction, TransactionPayload},
};
use starcoin_vm_types::genesis_config::StdlibVersion;
use starcoin_vm_types::token::stc;
use starcoin_vm_types::transaction::authenticator::AuthenticationKey;
use starcoin_vm_types::transaction::ScriptFunction;
use std::sync::Arc;

use starcoin_metrics::metrics::VMMetrics;
use starcoin_metrics::Registry;
use starcoin_vm_runtime::starcoin_vm::StarcoinVM;

use crate::create_transaction;

const INIT_ACCOUNT_BALANCE: u64 = 40_000_000_000;

struct AccountData {
    public_key: Ed25519PublicKey,
    private_key: Ed25519PrivateKey,
    address: AccountAddress,
    sequence_number: u64,
}

fn create_transaction_with_sender(
    sender: &AccountData,
    sequence_number: u64,
    program: ScriptFunction,
    expiration_timestamp_secs: u64,
    net: &ChainNetwork,
) -> Transaction {
    let raw_txn = RawUserTransaction::new_with_default_gas_token(
        sender.address,
        sequence_number,
        TransactionPayload::ScriptFunction(program),
        40_000_000,
        1,
        expiration_timestamp_secs,
        net.chain_id(),
    );

    let signed_txn = raw_txn
        .sign(&sender.private_key, sender.public_key.clone())
        .expect("sign with private key failed")
        .into_inner();
    Transaction::UserTransaction(signed_txn)
}

struct TransactionGenerator {
    accounts: Vec<AccountData>,
    net: ChainNetwork,
}

impl TransactionGenerator {
    fn new(num_accounts: usize, net: ChainNetwork) -> Self {
        let seed = [1u8; 32];
        let mut rng = StdRng::from_seed(seed);

        let mut accounts = Vec::with_capacity(num_accounts);
        for _i in 0..num_accounts {
            let private_key = Ed25519PrivateKey::generate(&mut rng);
            let public_key = private_key.public_key();
            let address = account_address::from_public_key(&public_key);
            let account = AccountData {
                public_key,
                private_key,
                address,
                sequence_number: 0,
            };
            accounts.push(account);
        }

        Self { accounts, net }
    }

    fn gen_create_account_transactions(&mut self) -> Vec<Transaction> {
        self.net.time_service().sleep(1000);
        let mut sequence: u64 = 0;

        let mut txns = Vec::with_capacity(self.accounts.len() + 1);
        for account in self.accounts.iter() {
            let txn = create_transaction(
                sequence,
                encode_create_account_script_function(
                    StdlibVersion::Latest,
                    stc::stc_type_tag(),
                    &account.address,
                    AuthenticationKey::ed25519(&account.public_key),
                    INIT_ACCOUNT_BALANCE as u128,
                ),
                self.net.time_service().now_secs() + sequence + 1,
                &self.net,
            );
            txns.push(txn);
            sequence += 1;
        }

        txns
    }

    fn gen_transfer_transactions(&mut self, txns_num: usize) -> Vec<Transaction> {
        self.net.time_service().sleep(1000);
        let mut txns = Vec::with_capacity(txns_num);
        for index in 0..self.accounts.len() / 2 {
            let sender_idx = 2 * index;
            let receiver_idx = 2 * index + 1;
            if receiver_idx >= self.accounts.len() {
                break;
            }
            let sender = &self.accounts[sender_idx];
            let receiver = &self.accounts[receiver_idx];
            let txn = create_transaction_with_sender(
                sender,
                self.accounts[sender_idx].sequence_number,
                encode_transfer_script_function(receiver.address, 1),
                self.net.time_service().now_secs() + self.accounts[sender_idx].sequence_number + 1,
                &self.net,
            );
            self.accounts[sender_idx].sequence_number += 1;
            txns.push(txn);
            if txns.len() >= txns_num {
                break;
            }
        }

        txns
    }
}

pub struct BenchmarkReport {
    concurrency_level: usize,
    txns: usize,
    exec_milliseconds: f64,
    tps: f64,
}

struct TransactionExecutor<'test, S> {
    chain_state: &'test S,
}

impl<'test, S: ChainStateReader + ChainStateWriter> TransactionExecutor<'test, S> {
    fn new(chain_state: &'test S) -> Self {
        Self { chain_state }
    }

    fn run(&mut self, txns: Vec<Transaction>) -> BenchmarkReport {
        let num_txns = txns.len();

        let registry = Registry::new();
        let vm_metrics = VMMetrics::register(&registry).ok();

        let _ =
            starcoin_executor::block_execute(self.chain_state, txns, u64::MAX, vm_metrics.clone())
                .expect("Execute txns fail.");
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
        let storage = Arc::new(Storage::new(StorageInstance::new_cache_instance()).unwrap());
        let chain_state = ChainStateDB::new(storage, None);
        let net = ChainNetwork::new_test();
        let genesis_txn = Genesis::build_genesis_transaction(&net).unwrap();
        let _ = Genesis::execute_genesis_txn(&chain_state, genesis_txn).unwrap();
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
        let mut generator = TransactionGenerator::new(max_txns_once * 2, self.net.clone());
        let txns = generator.gen_create_account_transactions();
        let mut executor = TransactionExecutor::new(&self.chain_state);
        let _ = executor.run(txns);

        // run serialize txns
        for txns_num in serialize_bench_txns.iter() {
            let txns = generator.gen_transfer_transactions(*txns_num);
            reports.push(executor.run(txns));
        }

        // this variable could only be set once, default is serialize, so we run serialize first.
        StarcoinVM::set_concurrency_level_once(num_cpus::get());
        assert_eq!(StarcoinVM::get_concurrency_level(), num_cpus::get());

        // run parallel txns
        for txns_num in parallel_bench_txns.iter() {
            let txns = generator.gen_transfer_transactions(*txns_num);
            reports.push(executor.run(txns));
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
