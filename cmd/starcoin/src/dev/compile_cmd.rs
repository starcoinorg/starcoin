// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::{bail, Result};
use scmd::{CommandAction, ExecContext};
use starcoin_move_compiler::command_line::parse_address;
use starcoin_move_compiler::shared::Address;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "compile")]
pub struct CompileOpt {
    #[structopt(short = "s", long = "sender", name = "sender address", help = "hex encoded string, like 0x0, 0x1", parse(try_from_str = parse_address))]
    sender: Option<Address>,

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
    type ReturnItem = PathBuf;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let sender = if let Some(sender) = ctx.opt().sender {
            sender
        } else {
            Address::new(ctx.state().default_account()?.address.into())
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
        let temp_dir = ctx.state().temp_dir();
        let source_file_path =
            starcoin_move_compiler::process_source_tpl_file(temp_dir, source_file_path, sender)?;
        let mut deps = stdlib::stdlib_files();
        // add extra deps
        deps.append(&mut ctx.opt().deps.clone());

        let targets = vec![source_file_path
            .to_str()
            .expect("path to str should success.")
            .to_owned()];
        let (file_texts, compile_units) =
            starcoin_move_compiler::move_compile_no_report(&targets, &deps, Some(sender))?;
        let mut compile_units = match compile_units {
            Err(e) => {
                let err =
                    starcoin_move_compiler::errors::report_errors_to_color_buffer(file_texts, e);
                bail!(String::from_utf8(err).unwrap())
            }
            Ok(r) => r,
        };
        let compile_result = compile_units.pop().unwrap();

        let mut txn_path = ctx
            .opt()
            .out_dir
            .clone()
            .unwrap_or_else(|| temp_dir.to_path_buf());

        txn_path.push(source_file_path.file_name().unwrap());
        txn_path.set_extension(stdlib::STAGED_EXTENSION);
        File::create(txn_path.clone())?.write_all(&compile_result.serialize())?;
        Ok(txn_path)
    }
}
