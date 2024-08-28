// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use clap::Parser;
// use move_cli::package::cli::handle_package_commands;
use move_cli::Move;
use move_core_types::errmap::ErrorMapping;
use move_package_manager::compatibility_check_cmd::{
    handle_compatibility_check, CompatibilityCheckCommand,
};
use move_package_manager::deployment::{handle_deployment, DeploymentCommand};
use move_package_manager::package::{handle_package_commands, PackageCommand};
use move_package_manager::release::{handle_release, Release};
use move_package_manager::{run_integration_test, IntegrationTestCommand};
use move_vm_test_utils::gas_schedule::CostTable;
use starcoin_config::genesis_config::G_LATEST_GAS_PARAMS;
use starcoin_vm_runtime::natives::starcoin_natives;
use starcoin_vm_types::on_chain_config::G_LATEST_INSTRUCTION_TABLE;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct CliOptions {
    #[clap(flatten)]
    move_args: Move,

    #[clap(subcommand)]
    cmd: Commands,
}

#[derive(Parser)]
pub enum Commands {
    /// Execute a package command. Executed in the current directory or the closest containing Move
    /// package.
    #[clap(name = "package")]
    Package {
        #[clap(subcommand)]
        cmd: PackageCommand,
    },
    /// Release the package.
    #[clap(name = "release")]
    Release(Release),
    /// TODO(simon) add subcommand to run static analyses on Move source or bytecode.
    /// Run integration tests in tests dir.
    #[clap(name = "integration-test", alias = "spectest")]
    IntegrationTest(IntegrationTestCommand),

    /// Check compatibility of modules comparing with remote chain state.
    #[clap(name = "check-compatibility")]
    CompatibilityCheck(CompatibilityCheckCommand),

    /// Deploy package to chain
    #[clap(name = "deploy")]
    Deploy(DeploymentCommand),
}

fn main() -> Result<()> {
    let _error_descriptions: ErrorMapping =
        bcs_ext::from_bytes(stdlib::ERROR_DESCRIPTIONS).expect("Decode err map failed");
    let args: CliOptions = CliOptions::parse();

    let move_args = &args.move_args;
    let gas_params = G_LATEST_GAS_PARAMS.clone();
    let natives = starcoin_natives(gas_params.natives);
    let _cost_table = CostTable {
        instruction_table: G_LATEST_INSTRUCTION_TABLE.clone(),
    };
    match args.cmd {
        Commands::IntegrationTest(cmd) => run_integration_test(args.move_args, cmd),
        Commands::Package { cmd } => handle_package_commands(natives, args.move_args, cmd),
        Commands::Release(release) => handle_release(move_args, release),
        Commands::CompatibilityCheck(cmd) => handle_compatibility_check(move_args, cmd),
        Commands::Deploy(cmd) => handle_deployment(move_args, cmd),
    }
}
