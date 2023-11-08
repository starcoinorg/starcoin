use clap::Parser;
use num_cpus;
use proptest::prelude::*;
use starcoin_language_e2e_tests::account_universe::P2PTransferGen;
use starcoin_transaction_benchmarks::transactions::TransactionBencher;

#[derive(Debug, Parser)]
pub struct ConcurrencyLevelOpt {
    /// concurrency level
    #[clap(long, short = 'n', use_delimiter = true)]
    pub concurrency_level: Vec<usize>,

    /// Transaction numbers
    #[clap(long, short = 't', use_delimiter = true)]
    pub txn_nums: Vec<usize>,

    /// Account numbers
    #[clap(long, short = 'a', use_delimiter = true)]
    pub account_nums: Vec<usize>,

    /// run parallel
    #[clap(long, short = 'p', parse(try_from_str), default_value = "true")]
    pub run_par: bool,

    /// run seq
    #[clap(long, short = 's', parse(try_from_str), default_value = "true")]
    pub run_seq: bool,
}

fn main() {
    let opt: ConcurrencyLevelOpt = ConcurrencyLevelOpt::parse();
    let default_num_accounts = 100;
    let default_num_transactions = 1_000;
    let concurrency_levels = opt.concurrency_level;
    let txns = opt.txn_nums;
    let account_nums = opt.account_nums;
    let mut run_par = opt.run_par;
    let run_seq = true;

    assert!(
        !concurrency_levels.is_empty(),
        "Concurrcy level array is empty!"
    );
    assert!(
        !txns.is_empty(),
        "Transaction numbers level array is empty!"
    );
    assert!(
        !account_nums.is_empty(),
        "Transaction numbers level array is empty!"
    );

    if !concurrency_levels.is_empty() {
        run_par = true;
    }

    // let acts = [2];
    // let txns = [1];
    // let num_warmups = 2;
    // let num_runs = 1;

    let bencher = TransactionBencher::new(any_with::<P2PTransferGen>((1_000, 1_000_000)));

    // let acts = [1000];
    //let txns = [10000, 50000, 100000];
    // let num_warmups = 2;
    // let num_runs = 10;

    println!(
        "num cpus = {}, run_seq: {}, run_seq: {}",
        num_cpus::get(),
        run_seq,
        run_seq
    );

    for concurrency_level in &concurrency_levels {
        let mut par_measurements = Vec::new();
        let mut seq_measurements = Vec::new();

        println!(
            "=========== concurrency_level:  {} started ===========",
            concurrency_level
        );

        for num_accounts in &account_nums {
            println!("=== accounts_num: {} started ===", num_accounts);
            for block_size in &txns {
                let (mut par_tps, mut seq_tps) = bencher.blockstm_benchmark(
                    *num_accounts,
                    *block_size,
                    run_par || (*concurrency_level > 1),
                    run_seq,
                    num_warmups,
                    num_runs,
                    *concurrency_level,
                );
                par_tps.sort();
                seq_tps.sort();
                par_measurements.push(par_tps);
                seq_measurements.push(seq_tps);
            }
            println!("=== accounts_num: {} completed ===", num_accounts);
        }

        let mut i = 0;
        for num_accounts in &account_nums {
            for block_size in &txns {
                println!(
                    "PARAMS: num_account = {}, block_size = {}",
                    *num_accounts, *block_size
                );

                let mut seq_tps = 0;
                let seq_measurement = &seq_measurements[i];
                let par_measurement = &par_measurements[i];
                if run_seq {
                    println!("Sequential TPS: {:?}", seq_measurement);
                    let mut seq_sum = 0;
                    for m in seq_measurement {
                        seq_sum += m;
                    }
                    seq_tps = seq_sum / seq_measurement.len();
                    println!("Avg Sequential TPS = {:?}", seq_tps,);
                }

                if run_par {
                    println!("Parallel TPS: {:?}", par_measurement);
                    let mut par_sum = 0;
                    for m in &par_measurements[i] {
                        par_sum += m;
                    }
                    let par_tps = par_sum / par_measurement.len();
                    println!("Avg Parallel TPS = {:?}", par_tps,);
                    if run_seq {
                        println!("Speed up {}x over sequential", par_tps / seq_tps);
                    }
                }
                i += 1;
            }
        }
        println!(
            "=========== concurrency_level:  {} finished ===========",
            concurrency_level
        );
    }
}
