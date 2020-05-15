use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::{bail, Result};
use move_lang::command_line::parse_address;
use move_lang::shared::Address;
use scmd::{CommandAction, ExecContext};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use structopt::StructOpt;
#[derive(Debug, StructOpt)]
#[structopt(name = "compile")]
pub struct CompileOpt {
    #[structopt(short = "s", long = "sender", name = "sender address", help = "hex encoded string, like 0x0, 0x1", parse(try_from_str = parse_address))]
    account_address: Option<Address>,

    #[structopt(short = "f", name = "source", help = "source file path")]
    source_file: String,

    #[structopt(
        short = "d",
        name = "dependency_path",
        long = "dep",
        help = "path of dependency used to build, support multi deps"
    )]
    deps: Vec<String>,

    #[structopt(short = "o", name = "out_dir", help = "out dir", parse(from_os_str))]
    out_dir: PathBuf,
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
        let address = ctx.opt().account_address;
        let source_file = ctx.opt().source_file.clone();

        let mut deps = stdlib::stdlib_files();
        // add extra deps
        deps.append(&mut ctx.opt().deps.clone());

        let targets = vec![source_file.clone()];
        let (file_texts, compile_units) =
            move_lang::move_compile_no_report(&targets, &deps, address)?;
        let mut compile_units = match compile_units {
            Err(e) => {
                let err = move_lang::errors::report_errors_to_color_buffer(file_texts, e);
                bail!(String::from_utf8(err).unwrap())
            }
            Ok(r) => r,
        };
        let compile_result = compile_units.pop().unwrap();

        let mut txn_path = ctx.opt().out_dir.clone();
        let source_file_path = Path::new(&source_file);
        txn_path.push(source_file_path.file_name().unwrap());
        txn_path.set_extension(stdlib::STAGED_EXTENSION);
        File::create(txn_path.clone())?.write_all(&compile_result.serialize())?;
        Ok(txn_path)
    }
}
