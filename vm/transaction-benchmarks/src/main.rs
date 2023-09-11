use num_cpus;
use proptest::prelude::*;
use starcoin_language_e2e_tests::account_universe::P2PTransferGen;
use starcoin_transaction_benchmarks::transactions::TransactionBencher;
use clap::Parser;

#[derive(Debug, Parser)]
#[clap(name = "concurrency level", about = "concurrency level")]
pub struct ConcurrencyLevelOpt {
    #[clap(long, short = 'n')]
    /// concurrency level
    pub concurrency_level: usize,
}

fn main() {
    let opt: ConcurrencyLevelOpt = ConcurrencyLevelOpt::parse();
    let default_num_accounts = 100;
    let default_num_transactions = 1_000;

    let bencher = TransactionBencher::new(
        any_with::<P2PTransferGen>((1_000, 1_000_000)),
        default_num_accounts,
        default_num_transactions,
    );

    let acts = [1000];
    let txns = [500000];
    let num_warmups = 2;
    let num_runs = 10;
    let num_threads = opt.concurrency_level;

    println!("num cpus = {}", num_cpus::get());

    let mut measurements = Vec::new();

    for block_size in txns {
        for num_accounts in acts {
            let mut times = bencher.manual_parallel(
                num_accounts,
                block_size,
                num_warmups,
                num_runs,
                num_threads,
            );
            times.sort();
            measurements.push(times);
        }
    }

    println!("CPUS = {}", num_cpus::get());

    let mut i = 0;
    for block_size in txns {
        for num_accounts in acts {
            println!(
                "PARAMS: num_account = {}, block_size = {}",
                num_accounts, block_size
            );
            println!("TPS: {:?}", measurements[i]);
            let mut sum = 0;
            for m in &measurements[i] {
                sum += m;
            }
            println!("AVG TPS = {:?}", sum / measurements[i].len());
            i = i + 1;
        }
        println!();
    }
}
