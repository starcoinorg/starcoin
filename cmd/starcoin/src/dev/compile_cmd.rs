// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::view::StringView;
use crate::StarcoinOpt;
use anyhow::{bail, ensure, format_err, Result};
use scmd::{CommandAction, ExecContext};
use starcoin_config::temp_path;
use starcoin_move_compiler::{compile_source_string_no_report, errors};
use starcoin_vm_types::account_address::AccountAddress;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use stdlib::restore_stdlib_in_dir;
use structopt::StructOpt;

/// Compile a module or script.
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
    deps: Option<Vec<String>>,

    #[structopt(short = "o", name = "out_dir", help = "out dir", parse(from_os_str))]
    out_dir: Option<PathBuf>,

    #[structopt(name = "source", help = "source file path")]
    source_file: String,

    /// Do not automatically run the bytecode verifier
    #[structopt(long = "no-verify")]
    pub no_verify: bool,
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

        let temp_path = temp_path();
        let mut deps = restore_stdlib_in_dir(temp_path.path())?;

        // add extra deps
        deps.append(&mut ctx.opt().deps.clone().unwrap_or_default());
        let (sources, compile_result) = compile_source_string_no_report(
            std::fs::read_to_string(source_file_path)
                .map_err(|e| {
                    format_err!("read source file({:?}) error: {:?}", source_file_path, e)
                })?
                .as_str(),
            &deps,
            sender,
        )?;

        let compile_result = if ctx.opt().no_verify {
            compile_result
        } else {
            compile_result.and_then(|units| {
                let (units, errors) = units.into_iter().map(|unit| unit.verify()).fold(
                    (vec![], vec![]),
                    |(mut units, mut errors), (unit, error)| {
                        units.push(unit);
                        errors.extend(error);
                        (units, errors)
                    },
                );
                if !errors.is_empty() {
                    Err(errors)
                } else {
                    Ok(units)
                }
            })
        };

        let compile_unit = match compile_result {
            Ok(mut c) => c
                .pop()
                .ok_or_else(|| anyhow::anyhow!("file should at least contain one compile unit"))?,
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

        let mut out_dir = ctx
            .opt()
            .out_dir
            .clone()
            .unwrap_or_else(|| ctx.state().temp_dir().to_path_buf());
        if !out_dir.exists() {
            std::fs::create_dir_all(out_dir.as_path())
                .map_err(|e| format_err!("make out_dir({:?}) error: {:?}", out_dir, e))?;
        }
        ensure!(out_dir.is_dir(), "out_dir should is a dir.");
        out_dir.push(source_file_path.file_name().unwrap());
        out_dir.set_extension(stdlib::COMPILED_EXTENSION);
        let mut file = File::create(out_dir.clone())
            .map_err(|e| format_err!("create file({:?} error: {:?})", out_dir, e))?;
        file.write_all(&compile_unit.serialize())
            .expect("write out file error");
        Ok(StringView {
            result: out_dir
                .to_str()
                .expect("out_dir to string should success")
                .to_string(),
        })
    }
}
