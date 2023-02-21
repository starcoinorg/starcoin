// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use move_binary_format::file_format_common::VERSION_4;
use move_binary_format::CompiledModule;
use move_cli::sandbox::utils::PackageContext;
use move_cli::Move;
use move_compiler::compiled_unit::{CompiledUnit, NamedCompiledModule};
use move_core_types::language_storage::TypeTag;
use move_core_types::transaction_argument::{convert_txn_args, TransactionArgument};
use starcoin_crypto::hash::PlainCryptoHash;
use starcoin_move_compiler::bytecode_transpose::ModuleBytecodeDowngrader;
use starcoin_types::transaction::parse_transaction_argument;
use starcoin_vm_types::language_storage::FunctionId;
use starcoin_vm_types::parser::parse_type_tag;
use starcoin_vm_types::transaction::{Module, Package, ScriptFunction};
use std::path::PathBuf;

pub const DEFAULT_RELEASE_DIR: &str = "release";

// XXX FIXME YSG, mpm release use v4, v6
#[derive(Parser)]
pub struct Release {
    #[clap(name = "move-version", long = "move-version", default_value="4", possible_values=&["4", "6"])]
    /// specify the move lang version for the release.
    /// currently, only v4, v6 are supported.
    language_version: u8,

    #[clap(name="release-dir", long, parse(from_os_str), default_value=DEFAULT_RELEASE_DIR)]
    /// dir to store released blob
    release_dir: PathBuf,

    #[clap(long = "function", name = "script-function")]
    /// init script function to execute, example: 0x123::MyScripts::init_script
    init_script: Option<FunctionId>,

    #[clap(
    short = 't',
    long = "type_tag",
    name = "type-tag",
    parse(try_from_str = parse_type_tag)
    )]
    /// type tags for the init script function
    type_tags: Option<Vec<TypeTag>>,

    #[clap(long = "arg", name = "transaction-args", parse(try_from_str = parse_transaction_argument))]
    /// args for the init script function
    args: Option<Vec<TransactionArgument>>,
}

pub fn handle_release(
    move_args: &Move,
    Release {
        language_version,
        mut release_dir,
        init_script,
        type_tags,
        args,
    }: Release,
) -> anyhow::Result<()> {
    let mut ms = vec![];
    let pkg_ctx = PackageContext::new(&move_args.package_path, &move_args.build_config)?;
    let pkg = pkg_ctx.package();
    let pkg_version = move_args
        .build_config
        .clone()
        .resolution_graph_for_package(
            move_args.package_path.as_ref().unwrap(),
            &mut std::io::stdout(),
        )
        .unwrap()
        .root_package
        .package
        .version;
    let pkg_name = pkg.compiled_package_info.package_name.as_str();
    println!("Packaging Modules:");
    for m in pkg.root_compiled_units.as_slice() {
        let m = module(&m.unit)?;
        println!("\t {}", m.self_id());
        // XXX FIXME YSG, mpm release
        let code = if language_version as u32 == VERSION_4 {
            ModuleBytecodeDowngrader::to_v4(m)?
        } else {
            let mut data = vec![];
            m.serialize(&mut data)?;
            data
        };
        ms.push(Module::new(code));
    }
    let init_script = match &init_script {
        Some(script) => {
            let type_tags = type_tags.unwrap_or_default();
            let args = args.unwrap_or_default();
            let script_function = script.clone();
            Some(ScriptFunction::new(
                script_function.module,
                script_function.function,
                type_tags,
                convert_txn_args(&args),
            ))
        }
        None => None,
    };

    let p = Package::new(ms, init_script)?;
    let blob = bcs_ext::to_bytes(&p)?;
    let release_path = {
        std::fs::create_dir_all(&release_dir)?;
        release_dir.push(format!(
            "{}.v{}.{}.{}.blob",
            pkg_name, pkg_version.0, pkg_version.1, pkg_version.2
        ));
        release_dir
    };
    std::fs::write(&release_path, blob)?;
    println!(
        "Release done: {}, package hash: {}",
        release_path.display(),
        p.crypto_hash()
    );
    Ok(())
}

pub fn module(unit: &CompiledUnit) -> anyhow::Result<&CompiledModule> {
    match unit {
        CompiledUnit::Module(NamedCompiledModule { module, .. }) => Ok(module),
        _ => anyhow::bail!("Found script in modules -- this shouldn't happen"),
    }
}
