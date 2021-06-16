// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::{bail, Result};
use scmd::{CommandAction, ExecContext};
use starcoin_crypto::hash::HashValue;
use starcoin_crypto::hash::PlainCryptoHash;
use starcoin_rpc_api::types::FunctionIdView;
use starcoin_types::transaction::{parse_transaction_argument, TransactionArgument};
use starcoin_vm_types::file_format::CompiledModule;
use starcoin_vm_types::transaction::ScriptFunction;
use starcoin_vm_types::transaction::{Module, Package};
use starcoin_vm_types::transaction_argument::convert_txn_args;
use starcoin_vm_types::{language_storage::TypeTag, parser::parse_type_tag};
use std::env::current_dir;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::{Path, PathBuf};
use structopt::StructOpt;

/// Build a modules package.
#[derive(Debug, StructOpt)]
#[structopt(name = "package")]
pub struct PackageOpt {
    #[structopt(
        short = "m",
        name = "module-file",
        long = "module",
        help = "path for module file, can be a folder, can be empty.",
        parse(from_os_str)
    )]
    module_file: Option<PathBuf>,

    #[structopt(
        long = "function",
        name = "script-function",
        help = "init script function to execute, example: 0x1::TransferScripts::peer_to_peer"
    )]
    init_script: Option<FunctionIdView>,

    #[structopt(
    short = "t",
    long = "type_tag",
    name = "type-tag",
    parse(try_from_str = parse_type_tag)
    )]
    /// type tags for the script
    type_tags: Option<Vec<TypeTag>>,

    #[structopt(long = "arg", name = "transaction-args", parse(try_from_str = parse_transaction_argument))]
    /// args for the script.
    args: Option<Vec<TransactionArgument>>,

    #[structopt(short = "o", name = "out-dir", help = "out dir", parse(from_os_str))]
    out_dir: Option<PathBuf>,

    #[structopt(short = "n", name = "package-name", long = "name")]
    /// package file name
    package_name: String,
}

pub struct PackageCmd;

impl CommandAction for PackageCmd {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = PackageOpt;
    type ReturnItem = HashValue;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let opt = ctx.opt();
        if let Some(module_file) = &opt.module_file {
            let mut compiled_modules = Vec::new();
            if module_file.is_file() {
                compiled_modules.push(read_module(module_file)?);
            } else if module_file.is_dir() {
                for entry in fs::read_dir(module_file)? {
                    let entry = entry?;
                    let path = entry.path();
                    compiled_modules.push(read_module(&path)?);
                }
            }
            let modules = compiled_modules
                .iter()
                .map(|m| {
                    let mut blob = vec![];
                    m.serialize(&mut blob)
                        .expect("serializing stdlib must work");
                    Module::new(blob)
                })
                .collect();

            let init_script = match &opt.init_script {
                Some(script) => {
                    let type_tags = opt.type_tags.clone().unwrap_or_default();
                    let args = opt.args.clone().unwrap_or_default();
                    let script_function = script.clone().0;
                    Some(ScriptFunction::new(
                        script_function.module,
                        script_function.function,
                        type_tags,
                        convert_txn_args(&args),
                    ))
                }
                None => None,
            };

            let package = Package::new(modules, init_script)?;

            let output_file = {
                let mut output_dir = opt.out_dir.clone().unwrap_or(current_dir()?);
                output_dir.push(opt.package_name.as_str());
                output_dir.set_extension("blob");
                output_dir
            };
            let mut file = File::create(output_file)?;
            let blob = bcs_ext::to_bytes(&package).unwrap();
            file.write_all(&blob).expect("write package file error");
            Ok(package.crypto_hash())
        } else {
            bail!("module file can not be empty.")
        }
    }
}

fn read_module(module_file: &Path) -> Result<CompiledModule> {
    if !module_file.is_file() {
        bail!("{:?} is not a file", module_file);
    }
    let mut bytes = vec![];
    File::open(module_file)?.read_to_end(&mut bytes)?;
    match CompiledModule::deserialize(bytes.as_slice()) {
        Err(e) => {
            bail!("invalid bytecode file, cannot deserialize as module, {}", e);
        }
        Ok(compiled_module) => Ok(compiled_module),
    }
}
