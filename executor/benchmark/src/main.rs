// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use starcoin_executor_benchmark::vm_exec_benchmark;

#[derive(Debug, Parser)]
struct Opt {
    #[clap(long, default_value = "200")]
    num_accounts: usize,

    #[clap(long, default_value = "1000000")]
    init_account_balance: u64,

    #[clap(long, default_value = "20")]
    block_size: usize,

    #[clap(long, default_value = "10")]
    num_transfer_blocks: usize,

    #[clap(long, default_value = "true")]
    bench_vm_exec: bool,

    #[clap(long, default_value = "true")]
    mute_info_log: bool,
}

fn main() {
    let opt = Opt::parse();

    if opt.mute_info_log {
        starcoin_logger::init_with_default_level("warn", None);
    } else {
        starcoin_logger::init();
    }

    if opt.bench_vm_exec {
        let mut manager = vm_exec_benchmark::BenchmarkManager::new();
        let reports = manager.run(&[2, 50, 100], &[2, 50, 100]);
        manager.pretty_print_reports(&reports);
    } else {
        rayon::ThreadPoolBuilder::new()
            .thread_name(|index| format!("rayon-global-{}", index))
            .build_global()
            .expect("Failed to build rayon global thread pool.");

        starcoin_executor_benchmark::run_benchmark(
            opt.num_accounts,
            opt.init_account_balance,
            opt.block_size,
            opt.num_transfer_blocks,
        );
    }
}
