// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use move_binary_format::file_format_common::VERSION_3;
use move_cli::package::cli::handle_package_commands;
use move_cli::sandbox::utils::PackageContext;
use move_cli::{experimental, package, sandbox, Move, DEFAULT_STORAGE_DIR};
use move_core_types::errmap::ErrorMapping;

use move_binary_format::CompiledModule;
use move_compiler::compiled_unit::{CompiledUnit, NamedCompiledModule};
use move_package_manager::{run_transactional_test, TransactionalTestCommand};
use starcoin_move_compiler::bytecode_transpose::ModuleBytecodeDowgrader;
use starcoin_vm_runtime::natives::starcoin_natives;
use starcoin_vm_types::transaction::{Module, Package};
use std::path::PathBuf;
use structopt::StructOpt;

pub const DEFAULT_RELEASE_DIR: &str = "release";
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
    Release {
        #[structopt(name = "move-version", long = "move-version", default_value="3", possible_values=&["3", "4"])]
        /// specify the move lang version for the release.
        /// currently, only v3, v4 are supported.
        language_version: u8,

        #[structopt(name="release-dir", long, parse(from_os_str), default_value=DEFAULT_RELEASE_DIR)]
        /// dir to store released blob
        release_dir: PathBuf,
    },
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

    // extra commands available can be added below
    #[structopt(name = "spectest")]
    TransactionalTest(TransactionalTestCommand),
}

fn main() -> Result<()> {
    let error_descriptions: ErrorMapping =
        bcs_ext::from_bytes(stdlib::ERROR_DESCRIPTIONS).expect("Decode err map failed");
    let args: CliOptions = CliOptions::from_args();
    let move_args = &args.move_args;
    let natives = starcoin_natives();
    match args.cmd {
        // Commands::Command(cmd) => move_cli::run_cli(
        //     starcoin_natives(),
        //     &error_descriptions,
        //     &args.move_args,
        //     &cmd,
        // ),
        Commands::TransactionalTest(cmd) => run_transactional_test(args.move_args, cmd),
        Commands::Package { cmd } => handle_package_commands(
            &move_args.package_path,
            move_args.build_config.clone(),
            &cmd,
            natives,
        ),
        Commands::Sandbox { storage_dir, cmd } => {
            cmd.handle_command(natives, &error_descriptions, move_args, &storage_dir)
        }
        Commands::Experimental { storage_dir, cmd } => cmd.handle_command(move_args, &storage_dir),
        Commands::Release {
            language_version,
            mut release_dir,
        } => {
            let mut ms = vec![];
            let pkg_ctx = PackageContext::new(&move_args.package_path, &move_args.build_config)?;
            let pkg = pkg_ctx.package();
            let pkg_version = move_args
                .build_config
                .clone()
                .resolution_graph_for_package(&move_args.package_path)
                .unwrap()
                .root_package
                .package
                .version;
            let pkg_name = pkg.compiled_package_info.package_name.as_str();
            println!("Packaging Modules:");
            for m in pkg.modules()? {
                let m = module(&m.unit)?;
                println!("\t {}", m.self_id());
                let code = if language_version as u32 == VERSION_3 {
                    ModuleBytecodeDowgrader::to_v3(m)?
                } else {
                    let mut data = vec![];
                    m.serialize(&mut data)?;
                    data
                };
                ms.push(Module::new(code));
            }
            let p = Package::new(ms, None)?;
            let package_bytes = bcs_ext::to_bytes(&p)?;
            let release_path = {
                std::fs::create_dir_all(&release_dir)?;
                release_dir.push(format!(
                    "{}.v{}.{}.{}.blob",
                    pkg_name, pkg_version.0, pkg_version.1, pkg_version.2
                ));
                release_dir
            };
            std::fs::write(&release_path, package_bytes)?;
            println!("Release done: {}", release_path.display());
            Ok(())
        }
    }
}
fn module(unit: &CompiledUnit) -> Result<&CompiledModule> {
    match unit {
        CompiledUnit::Module(NamedCompiledModule { module, .. }) => Ok(module),
        _ => anyhow::bail!("Found script in modules -- this shouldn't happen"),
    }
}
