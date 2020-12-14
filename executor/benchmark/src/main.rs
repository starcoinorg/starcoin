// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(long, default_value = "200")]
    num_accounts: usize,

    #[structopt(long, default_value = "1000000")]
    init_account_balance: u64,

    #[structopt(long, default_value = "20")]
    block_size: usize,

    #[structopt(long, default_value = "10")]
    num_transfer_blocks: usize,
}

fn main() {
    let opt = Opt::from_args();

    logger::init();

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
