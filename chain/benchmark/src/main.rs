// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use structopt::StructOpt;
use starcoin_chain_benchmark::ChainBencher;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(
    name = "number",
    long,
    short = "n",
    help = "block number"
    )]
    number: Option<u64>,
}

fn main() {
    let opt = Opt::from_args();
    let bench = ChainBencher::new(opt.number);
    bench.execute();
}
