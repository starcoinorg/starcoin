// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::view::StringView;
use crate::StarcoinOpt;
use anyhow::{bail, Result};
use scmd::{CommandAction, ExecContext};
use starcoin_move_compiler::{compile_source_string_no_report, errors};
use starcoin_vm_types::account_address::AccountAddress;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "compile")]
pub struct CompileOpt {
    #[structopt(
        short = "s",
        long = "sender",
        name = "sender address",
        help = "hex encoded string, like 0x0, 0x1"
    )]
    sender: Option<AccountAddress>,

    #[structopt(
        short = "d",
        name = "dependency_path",
        long = "dep",
        help = "path of dependency used to build, support multi deps"
    )]
    deps: Vec<String>,

    #[structopt(short = "o", name = "out_dir", help = "out dir", parse(from_os_str))]
    out_dir: Option<PathBuf>,

    #[structopt(name = "source", help = "source file path")]
    source_file: String,
}

pub struct CompileCommand;

impl CommandAction for CompileCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = CompileOpt;
    type ReturnItem = StringView;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let sender = if let Some(sender) = ctx.opt().sender {
            sender
        } else {
            ctx.state().default_account()?.address
        };
        let source_file = ctx.opt().source_file.as_str();
        let source_file_path = Path::new(source_file);
        let ext = source_file_path
            .extension()
            .map(|os_str| os_str.to_str().expect("file extension should is utf str"))
            .unwrap_or_else(|| "");
        //TODO support compile dir.
        if ext != starcoin_move_compiler::MOVE_EXTENSION {
            bail!("Only support compile *.move file.")
        }
        let mut deps = stdlib::stdlib_files();
        // add extra deps
        deps.append(&mut ctx.opt().deps.clone());
        let (sources, compile_result) = compile_source_string_no_report(
            std::fs::read_to_string(source_file_path)?.as_str(),
            &deps,
            sender,
        )?;
        let compile_unit = match compile_result {
            Ok(c) => c,
            Err(e) => {
                eprintln!(
                    "{}",
                    String::from_utf8_lossy(
                        errors::report_errors_to_color_buffer(sources, e).as_slice()
                    )
                );
                bail!("compile error")
            }
        };

        let mut txn_path = ctx
            .opt()
            .out_dir
            .clone()
            .unwrap_or_else(|| ctx.state().temp_dir().to_path_buf());

        txn_path.push(source_file_path.file_name().unwrap());
        txn_path.set_extension(stdlib::COMPILED_EXTENSION);
        let mut file = File::create(txn_path.clone()).expect("unable create out file");
        file.write_all(&compile_unit.serialize())
            .expect("write out file error");
        Ok(StringView {
            result: txn_path.to_str().unwrap().to_string(),
        })
    }
}
