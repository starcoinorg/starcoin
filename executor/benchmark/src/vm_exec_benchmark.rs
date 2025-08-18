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

// vm2
use starcoin_vm2_genesis::{build_and_execute_genesis_transaction, execute_genesis_transaction};
use starcoin_vm2_statedb::ChainStateDB as ChainStateDBVM2;
use starcoin_vm2_statedb::ChainStateReader as ChainStateReaderVM2;
use starcoin_vm2_statedb::ChainStateWriter as ChainStateWriterVM2;
use starcoin_vm2_test_helper::txn::create_account_txn_sent_as_association;
use starcoin_vm2_transaction_builder::build_transfer_txn as build_transfer_txn_vm2;
use starcoin_vm2_types::account::Account as AccountVM2;
use starcoin_vm2_types::account_address as account_address_vm2;
use starcoin_vm2_types::genesis_config::ChainId as ChainIdVM2;
use starcoin_vm2_types::transaction::SignedUserTransaction as SignedUserTransactionVM2;
use starcoin_vm2_types::transaction::Transaction as TransactionVM2;
use starcoin_vm2_executor;

// use starcoin_vm2_types:: as stc_vm2;

use crate::create_transaction;

const INIT_ACCOUNT_BALANCE: u64 = 40_000_000_000;
const MAX_GAS: u64 = 10_000_000;
const GAS_PER_UNIT: u64 = 1;
const TRANSFER_AMOUNT: u128 = 1;

struct AccountDataVM1 {
    public_key: Ed25519PublicKey,
    private_key: Ed25519PrivateKey,
    address: AccountAddress,
    sequence_number: u64,
}

struct AccountDataVM2 {
    account: AccountVM2,
    sequence_number: u64,
}

pub struct BenchmarkReport {
    concurrency_level: usize,
    txns: usize,
    exec_milliseconds: f64,
    tps: f64,
}
struct TransactionGeneratorVM1 {
    accounts: Vec<AccountDataVM1>,
    net: ChainNetwork,
}

struct TransactionGeneratorVM2 {
    accounts: Vec<AccountDataVM2>,
    net: ChainNetwork,
}

struct TransactionExecutorVM1<'test, S> {
    chain_state: &'test S,
}

struct TransactionExecutorVM2<'test, S> {
    chain_state: &'test S,
}

impl TransactionGeneratorVM1 {
    fn new(num_accounts: usize, net: ChainNetwork) -> Self {
        let seed = [1u8; 32];
        let mut rng = StdRng::from_seed(seed);

        let mut accounts = Vec::with_capacity(num_accounts);
        for _i in 0..num_accounts {
            let private_key = Ed25519PrivateKey::generate(&mut rng);
            let public_key = private_key.public_key();
            let address = account_address::from_public_key(&public_key);
            let account = AccountDataVM1 {
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

        let mut txns = Vec::with_capacity(self.accounts.len() + 1);
        for (sequence, account) in (0_u64..).zip(self.accounts.iter()) {
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
            let txn = TransactionGeneratorVM1::create_transaction_with_sender(
                sender,
                self.accounts[sender_idx].sequence_number,
                encode_transfer_script_function(receiver.address, TRANSFER_AMOUNT),
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

    fn create_transaction_with_sender(
        sender: &AccountDataVM1,
        sequence_number: u64,
        program: ScriptFunction,
        expiration_timestamp_secs: u64,
        net: &ChainNetwork,
    ) -> Transaction {
        let raw_txn = RawUserTransaction::new_with_default_gas_token(
            sender.address,
            sequence_number,
            TransactionPayload::ScriptFunction(program),
            MAX_GAS,
            GAS_PER_UNIT,
            expiration_timestamp_secs,
            net.chain_id(),
        );

        let signed_txn = raw_txn
            .sign(&sender.private_key, sender.public_key.clone())
            .expect("sign with private key failed")
            .into_inner();
        Transaction::UserTransaction(signed_txn)
    }
}

impl<'test, S: ChainStateReader + ChainStateWriter> TransactionExecutorVM1<'test, S> {
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

impl TransactionGeneratorVM2 {
    fn new(num_accounts: usize, net: ChainNetwork) -> Self {
        let mut accounts = Vec::with_capacity(num_accounts);
        for _i in 0..num_accounts {
            let mut account = AccountDataVM2 {
                account: AccountVM2::new(),
                sequence_number: 0,
            };
            account.account.addr = account_address_vm2::from_public_key(
                &account.account.public_key().as_single().unwrap(),
            );
            accounts.push(account);
        }

        Self { accounts, net }
    }

    fn gen_create_account_transactions(&mut self) -> Vec<TransactionVM2> {
        self.net.time_service().sleep(1000);

        let mut txns = Vec::with_capacity(self.accounts.len() + 1);
        for (sequence, account) in (0_u64..).zip(self.accounts.iter()) {
            let txn = create_account_txn_sent_as_association(
                &account.account,
                sequence,
                INIT_ACCOUNT_BALANCE as u128,
                self.net.time_service().now_secs() + sequence + 1,
                &self.net,
            );
            txns.push(TransactionVM2::UserTransaction(txn));
        }

        txns
    }

    fn gen_transfer_transactions(&mut self, txns_num: usize) -> Vec<TransactionVM2> {
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
            let raw = build_transfer_txn_vm2(
                sender.account.addr,
                receiver.account.addr,
                self.accounts[sender_idx].sequence_number,
                TRANSFER_AMOUNT,
                GAS_PER_UNIT,
                MAX_GAS,
                self.net.time_service().now_secs() + self.accounts[sender_idx].sequence_number + 1,
                ChainIdVM2::new(self.net.chain_id().id()),
            );
            let signature = sender
                .account
                .private_key()
                .sign(&raw)
                .expect("sign transaction failed");
            let signed_txn = SignedUserTransactionVM2::new(raw, signature);
            self.accounts[sender_idx].sequence_number += 1;
            txns.push(TransactionVM2::UserTransaction(signed_txn));
            if txns.len() >= txns_num {
                break;
            }
        }

        txns
    }
}

impl<'test, S: ChainStateReaderVM2 + ChainStateWriterVM2 + Sync> TransactionExecutorVM2<'test, S> {
    fn new(chain_state: &'test S) -> Self {
        Self { chain_state }
    }

    fn run(&mut self, txns: Vec<TransactionVM2>) -> BenchmarkReport {
        let num_txns = txns.len();

        let registry = Registry::new();
        let vm_metrics = VMMetrics::register(&registry).ok();

        let start_time = std::time::Instant::now();
        let _ = starcoin_vm2_executor::block_execute(
            self.chain_state,
            txns,
            u64::MAX,
            vm_metrics.clone(),
        )
        .expect("Execute txns fail.");
        let elapsed = start_time.elapsed();

        self.chain_state.flush().expect("flush state should be ok");

        let exec_milliseconds = elapsed.as_secs_f64() * 1000.0;
        let tps = if elapsed.as_secs_f64() > 0.0 {
            num_txns as f64 / elapsed.as_secs_f64()
        } else {
            0.0
        };

        BenchmarkReport {
            concurrency_level: 1, // VM2 doesn't have concurrency level concept like VM1
            txns: num_txns,
            exec_milliseconds,
            tps,
        }
    }
}

pub struct BenchmarkManager {
    vm1: Option<BenchmarkManagerVM1>,
    vm2: Option<BenchmarkManagerVM2>,
    bench_vm2: bool,
}

impl BenchmarkManager {
    pub fn new(bench_vm2: bool) -> Self {
        if bench_vm2 {
            Self {
                vm1: None,
                vm2: Some(BenchmarkManagerVM2::new()),
                bench_vm2: true,
            }
        } else {
            Self {
                vm1: Some(BenchmarkManagerVM1::new()),
                vm2: None,
                bench_vm2: false,
            }
        }
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

    pub fn run(
        &mut self,
        serialize_bench_txns: &[usize],
        parallel_bench_txns: &[usize],
    ) -> Vec<BenchmarkReport> {
        if self.bench_vm2 {
            if let Some(ref mut vm2) = self.vm2 {
                vm2.run(serialize_bench_txns, parallel_bench_txns)
            } else {
                panic!("BenchmarkManagerVM2 is not initialized.");
            }
        } else {
            if let Some(ref mut vm1) = self.vm1 {
                vm1.run(serialize_bench_txns, parallel_bench_txns)
            } else {
                panic!("BenchmarkManagerVM1 is not initialized.");
            }
        }
    }
}

struct BenchmarkManagerVM1 {
    chain_state: ChainStateDB,
    net: ChainNetwork,
}

struct BenchmarkManagerVM2 {
    chain_state: ChainStateDBVM2,
    net: ChainNetwork,
}

impl BenchmarkManagerVM2 {
    pub fn new() -> Self {
        let chain_state = ChainStateDBVM2::mock();
        let net = ChainNetwork::new_test();
        let (genesis_txn, _) =
            build_and_execute_genesis_transaction(net.chain_id().id(), net.genesis_config2());
        // execute_genesis_txn(&chain_state, genesis_txn).unwrap();
        execute_genesis_transaction(&chain_state, TransactionVM2::UserTransaction(genesis_txn))
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
        let mut generator = TransactionGeneratorVM2::new(max_txns_once * 2, self.net.clone());
        let txns = generator.gen_create_account_transactions();
        let mut executor = TransactionExecutorVM2::new(&self.chain_state);
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
}

impl BenchmarkManagerVM1 {
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
        let mut generator = TransactionGeneratorVM1::new(max_txns_once * 2, self.net.clone());
        let txns = generator.gen_create_account_transactions();
        let mut executor = TransactionExecutorVM1::new(&self.chain_state);
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
}
