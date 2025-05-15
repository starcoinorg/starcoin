// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;

use crate::{view::TransactionOptions, view_vm2::ExecuteResultView, CliState};
use anyhow::{bail, Result};
use clap::Parser;
use scmd::{CommandAction, ExecContext};
use starcoin_config::StarcoinOpt;
use starcoin_move_compiler::load_bytecode_file;
use starcoin_vm2_vm_types::{
    language_storage::TypeTag,
    parser::parse_type_tag,
    transaction::{Script, TransactionPayload},
    transaction_argument::{convert_txn_args, TransactionArgument},
};

use starcoin_vm2_types::transaction::parse_transaction_argument_advance;

/// Execute a script
#[derive(Debug, Parser)]
#[clap(name = "execute-script")]
pub struct ExecuteScriptOpt {
    #[clap(
    short = 't',
    long = "type_tag",
    name = "type-tag",
    help = "can specify multi type_tag",
    parse(try_from_str = parse_type_tag)
    )]
    type_tags: Option<Vec<TypeTag>>,

    #[clap(long = "arg", name = "transaction-args", help = "can specify multi arg", parse(try_from_str = parse_transaction_argument_advance))]
    args: Option<Vec<TransactionArgument>>,

    #[clap(flatten)]
    transaction_opts: TransactionOptions,

    #[clap(name = "mv_file", parse(from_os_str))]
    /// bytecode file of the script to execute.
    mv_file: PathBuf,
}

pub struct ExecuteScriptCommand;

impl CommandAction for ExecuteScriptCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = ExecuteScriptOpt;
    type ReturnItem = ExecuteResultView;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let opt = ctx.opt();
        let type_tags = opt.type_tags.clone().unwrap_or_default();
        let args = opt.args.clone().unwrap_or_default();
        let bytedata = { load_bytecode_file(opt.mv_file.as_path())? };
        let txn_payload = match bytedata {
            // script
            (bytecode, true) => {
                let script = Script::new(bytecode, type_tags, convert_txn_args(&args));
                TransactionPayload::Script(script)
            }
            _ => {
                bail!("bytecode is not a script!");
            }
        };
        ctx.state()
            .vm2()?
            .build_and_execute_transaction(opt.transaction_opts.clone(), txn_payload)
    }
}
