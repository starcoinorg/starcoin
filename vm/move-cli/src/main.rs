// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use move_cli::{Command, Move};
use move_core_types::errmap::ErrorMapping;
use move_package_manager::{run_transactional_test, TransactionalTestCommand};
use starcoin_vm_runtime::natives::starcoin_natives;
use structopt::StructOpt;

#[derive(StructOpt)]
pub struct CliOptions {
    #[structopt(flatten)]
    move_args: Move,

    #[structopt(subcommand)]
    cmd: Commands,
}

#[derive(StructOpt)]
pub enum Commands {
    #[structopt(flatten)]
    Command(Command),
    // extra commands available can be added below
    #[structopt(flatten)]
    TransactionalTest(TransactionalTestCommand),
}

fn main() -> Result<()> {
    let error_descriptions: ErrorMapping =
        bcs_ext::from_bytes(stdlib::ERROR_DESCRIPTIONS).expect("Decode err map failed");
    let args = CliOptions::from_args();
    match args.cmd {
        Commands::Command(cmd) => move_cli::run_cli(
            starcoin_natives(),
            &error_descriptions,
            &args.move_args,
            &cmd,
        ),
        Commands::TransactionalTest(cmd) => run_transactional_test(args.move_args, cmd),
    }
}
