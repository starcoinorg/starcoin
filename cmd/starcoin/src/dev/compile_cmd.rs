// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::view::StringView;
use crate::StarcoinOpt;
use anyhow::{bail, Result};
use scmd::{CommandAction, ExecContext};
use starcoin_move_compiler::command_line::parse_address;
use starcoin_move_compiler::shared::Address;
use starcoin_types::account_address::AccountAddress;
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
    type ReturnItem = StringView;

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
        let mut deps = stdlib::stdlib_files();
        // add extra deps
        deps.append(&mut ctx.opt().deps.clone());
        let compile_result = starcoin_move_compiler::compile_source_string(
            std::fs::read_to_string(source_file_path)
                .expect("read file error")
                .as_str(),
            &deps,
            AccountAddress::new(sender.to_u8()),
        )?;

        let mut txn_path = ctx
            .opt()
            .out_dir
            .clone()
            .unwrap_or_else(|| ctx.state().temp_dir().to_path_buf());

        txn_path.push(source_file_path.file_name().unwrap());
        txn_path.set_extension(stdlib::STAGED_EXTENSION);
        let mut file = File::create(txn_path.clone()).expect("unable create out file");
        file.write_all(&compile_result.serialize())
            .expect("write out file error");
        Ok(StringView {
            result: txn_path.to_str().unwrap().to_string(),
        })
    }
}
