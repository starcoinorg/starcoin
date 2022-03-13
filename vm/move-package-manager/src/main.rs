// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use move_cli::package::cli::handle_package_commands;
use move_cli::{experimental, package, sandbox, Move, DEFAULT_STORAGE_DIR};
use move_core_types::errmap::ErrorMapping;
use move_package_manager::compatibility_check_cmd::{
    handle_compatibility_check, CompatibilityCheckCommand,
};
use move_package_manager::releasement::{handle_release, Releasement};
use move_package_manager::{run_transactional_test, TransactionalTestCommand};
use starcoin_config::genesis_config;
use starcoin_vm_runtime::natives::starcoin_natives;
use std::path::PathBuf;
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
    /// Execute a package command. Executed in the current directory or the closest containing Move
    /// package.
    #[structopt(name = "package")]
    Package {
        #[structopt(subcommand)]
        cmd: package::cli::PackageCommand,
    },
    /// Release the package.
    #[structopt(name = "release")]
    Release(Releasement),
    /// Execute a sandbox command.
    #[structopt(name = "sandbox")]
    Sandbox {
        /// Directory storing Move resources, events, and module bytecodes produced by module publishing
        /// and script execution.
        #[structopt(long, default_value = DEFAULT_STORAGE_DIR, parse(from_os_str))]
        storage_dir: PathBuf,
        #[structopt(subcommand)]
        cmd: sandbox::cli::SandboxCommand,
    },
    /// (Experimental) Run static analyses on Move source or bytecode.
    #[structopt(name = "experimental")]
    Experimental {
        /// Directory storing Move resources, events, and module bytecodes produced by module publishing
        /// and script execution.
        #[structopt(long, default_value = DEFAULT_STORAGE_DIR, parse(from_os_str))]
        storage_dir: PathBuf,
        #[structopt(subcommand)]
        cmd: experimental::cli::ExperimentalCommand,
    },

    /// Run transaction tests in spectests dir.
    #[structopt(name = "spectest")]
    TransactionalTest(TransactionalTestCommand),
    /// Check compatibility of modules comparing with remote chain chate.
    #[structopt(name = "check-compatibility")]
    CompatibilityCheck(CompatibilityCheckCommand),
}

fn main() -> Result<()> {
    let error_descriptions: ErrorMapping =
        bcs_ext::from_bytes(stdlib::ERROR_DESCRIPTIONS).expect("Decode err map failed");
    let args: CliOptions = CliOptions::from_args();
    let move_args = &args.move_args;
    let natives = starcoin_natives();
    match args.cmd {
        Commands::TransactionalTest(cmd) => run_transactional_test(args.move_args, cmd),
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
