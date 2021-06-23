// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::view::StringView;
use crate::StarcoinOpt;
use anyhow::{bail, ensure, format_err, Result};
use scmd::{CommandAction, ExecContext};
use starcoin_crypto::hash::PlainCryptoHash;
use starcoin_rpc_api::types::FunctionIdView;
use starcoin_types::transaction::{parse_transaction_argument, TransactionArgument};
use starcoin_vm_types::transaction::ScriptFunction;
use starcoin_vm_types::transaction::{Module, Package};
use starcoin_vm_types::transaction_argument::convert_txn_args;
use starcoin_vm_types::{language_storage::TypeTag, parser::parse_type_tag};
use std::env::current_dir;
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
        name = "mv-file-or-dir",
        help = "path for move bytecode file, can be a folder.",
        parse(from_os_str)
    )]
    mv_file_or_dir: PathBuf,

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
    /// package file name, if absent, use file hash as name.
    package_name: Option<String>,
}

pub struct PackageCmd;

impl CommandAction for PackageCmd {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = PackageOpt;
    type ReturnItem = StringView;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let opt = ctx.opt();
        let mv_file_or_dir = opt.mv_file_or_dir.as_path();
        ensure!(
            mv_file_or_dir.exists(),
            "file {:?} not exist",
            mv_file_or_dir
        );
        let modules = if mv_file_or_dir.is_file() {
            vec![read_module(mv_file_or_dir)?]
        } else {
            starcoin_move_compiler::utils::iterate_directory(mv_file_or_dir)
                .map(|path| read_module(path.as_path()))
                .collect::<Result<Vec<Module>>>()?
        };

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
            output_dir.push(
                opt.package_name
                    .clone()
                    .unwrap_or_else(|| package.crypto_hash().to_string()),
            );
            output_dir.set_extension("blob");
            output_dir
        };
        let mut file = File::create(output_file.as_path())?;
        let blob = bcs_ext::to_bytes(&package)?;
        file.write_all(&blob)
            .map_err(|e| format_err!("write package file {:?} error:{:?}", output_file, e))?;
        Ok(StringView {
            result: output_file.to_string_lossy().to_string(),
        })
    }
}

fn read_module(module_file: &Path) -> Result<Module> {
    if !module_file.is_file() {
        bail!("{:?} is not a file", module_file);
    }
    let mut bytes = vec![];
    File::open(module_file)?.read_to_end(&mut bytes)?;
    Ok(Module::new(bytes))
}
