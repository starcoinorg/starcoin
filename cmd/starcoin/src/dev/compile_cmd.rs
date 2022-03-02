// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::{bail, ensure, format_err, Result};
use scmd::{CommandAction, ExecContext};
use starcoin_move_compiler::move_command_line_common::files::{
    MOVE_COMPILED_EXTENSION, MOVE_EXTENSION,
};
use starcoin_move_compiler::shared::Flags;
use starcoin_move_compiler::{
    compile_source_string_no_report, starcoin_framework_named_addresses, Compiler,
};
use starcoin_vm_types::account_address::AccountAddress;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use stdlib::stdlib_files;
use structopt::StructOpt;

/// Compile module or script, support compile source dir.
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

    #[structopt(name = "source_file_or_dir", help = "source file path")]
    source_file_or_dir: PathBuf,

    /// Do not automatically run the bytecode verifier
    #[structopt(long = "no-verify")]
    pub no_verify: bool,
}

pub struct CompileCommand;

impl CommandAction for CompileCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = CompileOpt;
    type ReturnItem = Vec<String>;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        eprintln!("WARNING: the command is deprecated in favor of move-package-manager, will be removed in next release.");
        let sender = if let Some(sender) = ctx.opt().sender {
            sender
        } else {
            ctx.state().default_account()?.address
        };
        let source_file_or_dir = ctx.opt().source_file_or_dir.as_path();

        ensure!(
            source_file_or_dir.exists(),
            "file {:?} not exist",
            source_file_or_dir
        );

        let mut deps = stdlib_files();

        // add extra deps
        deps.append(&mut ctx.opt().deps.clone().unwrap_or_default());
        let (sources, compile_result) = if source_file_or_dir.is_file() {
            let ext = source_file_or_dir
                .extension()
                .map(|os_str| os_str.to_string_lossy().to_string())
                .unwrap_or_default();
            if ext != MOVE_EXTENSION {
                bail!("{:?} not a move file", source_file_or_dir)
            }
            compile_source_string_no_report(
                std::fs::read_to_string(source_file_or_dir)
                    .map_err(|e| {
                        format_err!("read source file({:?}) error: {:?}", source_file_or_dir, e)
                    })?
                    .as_str(),
                &deps,
                sender,
            )?
        } else {
            let targets = vec![source_file_or_dir.to_string_lossy().to_string()];
            Compiler::new(&targets, &deps)
                .set_named_address_values(starcoin_framework_named_addresses())
                .set_flags(Flags::empty().set_sources_shadow_deps(true))
                .build()?
        };

        let compile_result = if ctx.opt().no_verify {
            compile_result.and_then(|v| if v.1.is_empty() { Ok(v.0) } else { Err(v.1) })
        } else {
            compile_result.and_then(|units| {
                let (units, errors) = units
                    .0
                    .into_iter()
                    .map(|unit| {
                        let dig = unit.verify();
                        (unit, dig)
                    })
                    .fold(
                        (vec![], units.1),
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

        let compile_units = match compile_result {
            Ok(c) => c,
            Err(e) => {
                eprintln!(
                    "{}",
                    String::from_utf8_lossy(
                        starcoin_move_compiler::diagnostics::report_diagnostics_to_color_buffer(
                            &sources, e,
                        )
                        .as_slice()
                    )
                );
                bail!("compile error")
            }
        };

        let out_dir = ctx
            .opt()
            .out_dir
            .clone()
            .unwrap_or_else(|| ctx.state().temp_dir().to_path_buf());
        if !out_dir.exists() {
            std::fs::create_dir_all(out_dir.as_path())
                .map_err(|e| format_err!("make out_dir({:?}) error: {:?}", out_dir, e))?;
        }
        ensure!(out_dir.is_dir(), "out_dir should is a dir.");
        let mut results = vec![];
        for unit in compile_units {
            let unit = unit.into_compiled_unit();
            let mut file_path = out_dir.join(unit.name().as_str());
            file_path.set_extension(MOVE_COMPILED_EXTENSION);
            let mut file = File::create(file_path.as_path())
                .map_err(|e| format_err!("create file({:?} error: {:?})", file_path, e))?;
            file.write_all(&unit.serialize())
                .map_err(|e| format_err!("write file({:?} error: {:?})", file_path, e))?;
            results.push(file_path.to_string_lossy().to_string());
        }
        Ok(results)
    }
}
