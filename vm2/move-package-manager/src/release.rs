// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use clap::{value_parser, Parser};
use move_binary_format::{file_format_common::VERSION_4, CompiledModule};
use move_cli::Move;
use move_compiler::compiled_unit::{CompiledUnit, NamedCompiledModule};
use move_core_types::{
    language_storage::TypeTag,
    transaction_argument::{convert_txn_args, TransactionArgument},
};
use starcoin_crypto::hash::PlainCryptoHash;
use starcoin_move_compiler::bytecode_transpose::ModuleBytecodeDowngrader;
use starcoin_types::transaction::parse_transaction_argument;
use starcoin_vm_types::{
    language_storage::FunctionId,
    parser::parse_type_tag,
    transaction::{Module, Package, ScriptFunction},
};
use std::path::PathBuf;

pub const DEFAULT_RELEASE_DIR: &str = "release";

#[derive(Parser)]
pub struct Release {
    #[arg(name = "move-version", long = "move-version", default_value_t = 6)]
    /// specify the move lang version for the release.
    /// currently, only v6 are supported.
    language_version: u8,

    #[arg(name="release-dir", long, value_parser = value_parser!(std::path::PathBuf), default_value=DEFAULT_RELEASE_DIR)]
    /// dir to store released blob
    release_dir: PathBuf,

    #[clap(long = "function", name = "script-function")]
    /// init script function to execute, example: 0x123::MyScripts::init_script
    init_script: Option<FunctionId>,

    #[arg(
    short = 't',
    long = "type_tag",
    name = "type-tag",
    value_parser = parse_type_tag
    )]
    /// type tags for the init script function
    type_tags: Option<Vec<TypeTag>>,

    #[arg(long = "arg", name = "transaction-args", value_parser = parse_transaction_argument)]
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
    unimplemented!()
    // let mut ms = vec![];
    // let package_path = match move_args.package_path {
    //     Some(_) => move_args.package_path.clone(),
    //     None => Some(std::env::current_dir()?),
    // };
    // let pkg_ctx = PackageContext::new(&package_path, &move_args.build_config)?;
    // let pkg = pkg_ctx.package();
    // let pkg_version = move_args
    //     .build_config
    //     .clone()
    //     .resolution_graph_for_package(package_path.as_ref().unwrap(), &mut std::io::stdout())
    //     .unwrap()
    //     .root_package
    //     .package
    //     .version;
    // let pkg_name = pkg.compiled_package_info.package_name.as_str();
    // println!("Packaging Modules:");
    // for m in pkg.root_compiled_units.as_slice() {
    //     let m = module(&m.unit)?;
    //     println!("\t {}", m.self_id());
    //     let code = if language_version as u32 == VERSION_4 {
    //         ModuleBytecodeDowngrader::to_v4(m)?
    //     } else {
    //         let mut data = vec![];
    //         m.serialize(&mut data)?;
    //         data
    //     };
    //     ms.push(Module::new(code));
    // }
    // let init_script = match &init_script {
    //     Some(script) => {
    //         let type_tags = type_tags.unwrap_or_default();
    //         let args = args.unwrap_or_default();
    //         let script_function = script.clone();
    //         Some(ScriptFunction::new(
    //             script_function.module,
    //             script_function.function,
    //             type_tags,
    //             convert_txn_args(&args),
    //         ))
    //     }
    //     None => None,
    // };
    //
    // let p = Package::new(ms, init_script)?;
    // let blob = bcs_ext::to_bytes(&p)?;
    // let release_path = {
    //     std::fs::create_dir_all(&release_dir)?;
    //     release_dir.push(format!(
    //         "{}.v{}.{}.{}.blob",
    //         pkg_name, pkg_version.0, pkg_version.1, pkg_version.2
    //     ));
    //     release_dir
    // };
    // std::fs::write(&release_path, blob)?;
    // println!(
    //     "Release done: {}, package hash: {}",
    //     release_path.display(),
    //     p.crypto_hash()
    // );
    // Ok(())
}

pub fn module(unit: &CompiledUnit) -> anyhow::Result<&CompiledModule> {
    match unit {
        CompiledUnit::Module(NamedCompiledModule { module, .. }) => Ok(module),
        _ => anyhow::bail!("Found script in modules -- this shouldn't happen"),
    }
}
