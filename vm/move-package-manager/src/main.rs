// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use clap::Parser;
use move_cli::package::cli::handle_package_commands;
use move_cli::{experimental, package, sandbox, Move, DEFAULT_STORAGE_DIR};
use move_core_types::errmap::ErrorMapping;
use move_package_manager::compatibility_check_cmd::{
    handle_compatibility_check, CompatibilityCheckCommand,
};
use move_package_manager::releasement::{handle_release, Releasement};
use move_package_manager::{run_integration_test, IntegrationTestCommand};
use starcoin_config::genesis_config;
use starcoin_vm_runtime::natives::starcoin_natives;
use std::path::PathBuf;

#[derive(Parser)]
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
        cmd: package::cli::PackageCommand,
    },
    /// Release the package.
    #[clap(name = "release")]
    Release(Releasement),
    /// Execute a sandbox command.
    #[clap(name = "sandbox")]
    Sandbox {
        /// Directory storing Move resources, events, and module bytecodes produced by module publishing
        /// and script execution.
        #[clap(long, default_value = DEFAULT_STORAGE_DIR, parse(from_os_str))]
        storage_dir: PathBuf,
        #[clap(subcommand)]
        cmd: sandbox::cli::SandboxCommand,
    },
    /// (Experimental) Run static analyses on Move source or bytecode.
    #[clap(name = "experimental")]
    Experimental {
        /// Directory storing Move resources, events, and module bytecodes produced by module publishing
        /// and script execution.
        #[clap(long, default_value = DEFAULT_STORAGE_DIR, parse(from_os_str))]
        storage_dir: PathBuf,
        #[clap(subcommand)]
        cmd: experimental::cli::ExperimentalCommand,
    },
    /// Run integration tests in tests dir.
    #[clap(name = "integration-test", alias = "spectest")]
    IntegrationTest(IntegrationTestCommand),

    /// Check compatibility of modules comparing with remote chain chate.
    #[clap(name = "check-compatibility")]
    CompatibilityCheck(CompatibilityCheckCommand),
}

fn main() -> Result<()> {
    let error_descriptions: ErrorMapping =
        bcs_ext::from_bytes(stdlib::ERROR_DESCRIPTIONS).expect("Decode err map failed");
    let args: CliOptions = CliOptions::parse();
    let move_args = &args.move_args;
    let natives = starcoin_natives();
    match args.cmd {
        Commands::IntegrationTest(cmd) => run_integration_test(args.move_args, cmd),
        Commands::Package { cmd } => handle_package_commands(
            &move_args.package_path,
            move_args.build_config.clone(),
            &cmd,
            natives,
        ),
        Commands::Sandbox { storage_dir, cmd } => cmd.handle_command(
            natives,
            &genesis_config::LATEST_GAS_SCHEDULE,
            &error_descriptions,
            move_args,
            &storage_dir,
        ),
        Commands::Experimental { storage_dir, cmd } => cmd.handle_command(move_args, &storage_dir),
        Commands::Release(releasement) => handle_release(move_args, releasement),
        Commands::CompatibilityCheck(cmd) => handle_compatibility_check(move_args, cmd),
    }
}
