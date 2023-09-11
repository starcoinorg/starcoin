use clap::Parser;
use num_cpus;
use proptest::prelude::*;
use starcoin_language_e2e_tests::account_universe::P2PTransferGen;
use starcoin_transaction_benchmarks::transactions::TransactionBencher;

#[derive(Debug, Parser)]
pub struct ConcurrencyLevelOpt {
    #[clap(long, short = 'n')]
    /// concurrency level
    pub concurrency_level: usize,
    #[clap(long, short = 'p')]
    /// run parallel
    pub run_par: bool,
    /// run seq
    #[clap(long, short = 's')]
    pub run_seq: bool,
}

fn main() {
    let opt: ConcurrencyLevelOpt = ConcurrencyLevelOpt::parse();
    let default_num_accounts = 100;
    let default_num_transactions = 1_000;
    let concurrency_level = opt.concurrency_level;
    let mut run_par = opt.run_par;
    let run_seq = true;

    if concurrency_level > 0 {
        run_par = true;
    }

    let bencher = TransactionBencher::new(
        any_with::<P2PTransferGen>((1_000, 1_000_000)),
        default_num_accounts,
        default_num_transactions,
    );

    let acts = [1000];
    let txns = [100000];
    let num_warmups = 2;
    let num_runs = 10;

    println!("num cpus = {}", num_cpus::get());

    let mut par_measurements = Vec::new();
    let mut seq_measurements = Vec::new();

    for block_size in txns {
        for num_accounts in acts {
            let (mut par_tps, mut seq_tps) = bencher.blockstm_benchmark(
                num_accounts,
                block_size,
                run_par,
                run_seq,
                num_warmups,
                num_runs,
                concurrency_level,
            );
            par_tps.sort();
            seq_tps.sort();
            par_measurements.push(par_tps);
            seq_measurements.push(seq_tps);
        }
    }

    println!("\nconcurrency_level = {}\n", concurrency_level);

    let mut i = 0;
    for block_size in txns {
        for num_accounts in acts {
            println!(
                "PARAMS: num_account = {}, block_size = {}",
                num_accounts, block_size
            );

            let mut seq_tps = 1;
            if run_seq {
                println!("Sequential TPS: {:?}", seq_measurements[i]);
                let mut seq_sum = 0;
                for m in &seq_measurements[i] {
                    seq_sum += m;
                }
                seq_tps = seq_sum / seq_measurements[i].len();
                println!("Avg Sequential TPS = {:?}", seq_tps,);
            }

            if run_par {
                println!("Parallel TPS: {:?}", par_measurements[i]);
                let mut par_sum = 0;
                for m in &par_measurements[i] {
                    par_sum += m;
                }
                let par_tps = par_sum / par_measurements[i].len();
                println!("Avg Parallel TPS = {:?}", par_tps,);
                if run_seq {
                    println!("Speed up {}x over sequential", par_tps / seq_tps);
                }
            }
            i += 1;
        }
        println!();
    }
}
