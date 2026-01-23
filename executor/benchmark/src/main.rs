// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use starcoin_executor_benchmark::vm_exec_benchmark::{self, VmStorage};

#[derive(Debug, Parser)]
struct Opt {
    #[clap(long)]
    num_accounts: Option<usize>,

    #[clap(long, default_value = "1000000")]
    init_account_balance: u64,

    #[clap(long, default_value = "20")]
    block_size: usize,

    #[clap(long, default_value = "10")]
    num_transfer_blocks: usize,

    #[clap(long, default_value = "true")]
    bench_vm_exec: bool,

    #[clap(long, default_value = "warn")]
    log_level: String,

    /// Only used in VM2 benchmark; override parallel executor concurrency
    #[clap(long)]
    vm_concurrency: Option<usize>,

    /// Only used in VM2 benchmark; choose storage backend
    #[clap(long, value_enum, default_value = "memory")]
    vm_storage: VmStorage,

    #[clap(long, default_value = "1,10")]
    serialize_txns: String,

    #[clap(long, default_value = "1,10")]
    parallel_txns: String,

    #[clap(long)]
    vm_exec_accounts: Option<usize>,

    #[clap(long)]
    vm_exec_concurrency: Option<usize>,
}

fn main() {
    let opt = Opt::parse();

    starcoin_logger::init_with_default_level(&opt.log_level, None);

    if opt.bench_vm_exec {
        let mut manager = vm_exec_benchmark::BenchmarkManager::new(opt.vm_storage);
        let serialize_txns: Vec<usize> = opt
            .serialize_txns
            .split(',')
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.trim().parse().expect("Invalid transaction count"))
            .collect();
        let parallel_txns: Vec<usize> = opt
            .parallel_txns
            .split(',')
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.trim().parse().expect("Invalid transaction count"))
            .collect();
        let vm_exec_accounts = opt.vm_exec_accounts.or(opt.num_accounts);
        let vm_exec_concurrency = opt.vm_exec_concurrency.or(opt.vm_concurrency);
        let reports = manager.run(
            &serialize_txns,
            &parallel_txns,
            vm_exec_accounts,
            vm_exec_concurrency,
        );
        manager.pretty_print_reports(&reports);
    } else {
        let num_accounts = opt.num_accounts.unwrap_or(200);
        rayon::ThreadPoolBuilder::new()
            .thread_name(|index| format!("rayon-global-{}", index))
            .build_global()
            .expect("Failed to build rayon global thread pool.");

        starcoin_executor_benchmark::run_benchmark(
            num_accounts,
            opt.init_account_balance,
            opt.block_size,
            opt.num_transfer_blocks,
        );
    }
}
