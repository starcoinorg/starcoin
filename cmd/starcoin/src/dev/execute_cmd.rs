// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::{bail, Result};
use scmd::{CommandAction, ExecContext};
use starcoin_crypto::hash::{HashValue, PlainCryptoHash};
use starcoin_move_compiler::shared::Address;
use starcoin_move_compiler::{
    command_line::parse_address, MOVE_COMPILED_EXTENSION, MOVE_EXTENSION,
};
use starcoin_rpc_client::RemoteStateReader;
use starcoin_state_api::AccountStateReader;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::transaction::{
    parse_transaction_argument, RawUserTransaction, Script, TransactionArgument,
};
use starcoin_vm_types::{language_storage::TypeTag, parser::parse_type_tag};
use std::fs::OpenOptions;
use std::io::Read;
use std::path::PathBuf;
use std::time::Duration;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "execute")]
pub struct ExecuteOpt {
    #[structopt(short = "s", long, parse(try_from_str = parse_address))]
    /// hex encoded string, like 0x1, 0x12
    sender: Option<Address>,

    #[structopt(
    short = "t",
    long = "type_tag",
    name = "type-tag",
    help = "can specify multi type_tag",
    parse(try_from_str = parse_type_tag)
    )]
    type_tags: Vec<TypeTag>,

    #[structopt(long = "arg", name = "transaction-args", help = "can specify multi arg", parse(try_from_str = parse_transaction_argument))]
    args: Vec<TransactionArgument>,

    #[structopt(
        name = "expiration_time",
        long = "timeout",
        default_value = "3000",
        help = "how long(in seconds) the txn stay alive"
    )]
    expiration_time: u64,

    #[structopt(
        short = "g",
        long = "max-gas",
        name = "max-gas-amount",
        default_value = "1000000",
        help = "max gas used to execute the script"
    )]
    max_gas_amount: u64,
    #[structopt(
        short = "p",
        long = "gas-price",
        name = "price of gas",
        default_value = "1",
        help = "gas price used to execute the script"
    )]
    gas_price: u64,
    #[structopt(
        short = "b",
        name = "blocking-mode",
        long = "blocking",
        help = "blocking wait txn mined"
    )]
    blocking: bool,

    #[structopt(name = "move_file", parse(from_os_str))]
    /// bytecode file or move script source file
    move_file: PathBuf,

    #[structopt(name = "dependency_path", long = "dep")]
    /// "path of dependency used to build, only "
    deps: Vec<String>,
}

pub struct ExecuteCommand;

impl CommandAction for ExecuteCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = ExecuteOpt;
    type ReturnItem = HashValue;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let opt = ctx.opt();

        let sender = if let Some(sender) = ctx.opt().sender {
            sender
        } else {
            Address::new(ctx.state().default_account()?.address.into())
        };

        let bytecode_path = ctx.opt().move_file.clone();

        let ext = bytecode_path
            .extension()
            .map(|os_str| os_str.to_str().expect("file extension should is utf8 str"))
            .unwrap_or_else(|| "");
        let bytecode = if ext == MOVE_EXTENSION {
            let temp_dir = ctx.state().temp_dir();
            let source_file_path = starcoin_move_compiler::process_source_tpl_file(
                temp_dir,
                bytecode_path.as_path(),
                sender,
            )?;
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
                    let err = starcoin_move_compiler::errors::report_errors_to_color_buffer(
                        file_texts, e,
                    );
                    bail!(String::from_utf8(err).unwrap())
                }
                Ok(r) => r,
            };
            let compile_result = compile_units.pop().unwrap();
            compile_result.serialize()
        } else if ext == MOVE_COMPILED_EXTENSION {
            let mut file = OpenOptions::new()
                .read(true)
                .write(false)
                .open(bytecode_path)?;
            let mut bytecode = vec![];
            file.read_to_end(&mut bytecode)?;
            let _compiled_script = match starcoin_vm_types::file_format::CompiledScript::deserialize(
                bytecode.as_slice(),
            ) {
                Err(e) => {
                    bail!("invalid bytecode file, cannot deserialize as script, {}", e);
                }
                Ok(s) => s,
            };
            bytecode
        } else {
            bail!("Only support *.move or *.mv file");
        };
        let args = opt.args.clone();
        let sender = AccountAddress::new(sender.to_u8());
        let client = ctx.state().client();
        let chain_state_reader = RemoteStateReader::new(client);
        let account_state_reader = AccountStateReader::new(&chain_state_reader);
        let account_resource = account_state_reader.get_account_resource(&sender)?;

        if account_resource.is_none() {
            bail!("address {} not exists on chain", &sender);
        }
        let account_resource = account_resource.unwrap();
        let expiration_time = Duration::from_secs(opt.expiration_time);
        let script_txn = RawUserTransaction::new_script(
            sender,
            account_resource.sequence_number(),
            Script::new(bytecode, opt.type_tags.clone(), args),
            opt.max_gas_amount,
            opt.gas_price,
            expiration_time,
        );

        let signed_txn = client.wallet_sign_txn(script_txn)?;
        let txn_hash = signed_txn.crypto_hash();
        let succ = client.submit_transaction(signed_txn)?;
        if let Err(e) = succ {
            bail!("execute-txn is reject by node, reason: {}", &e)
        }
        println!("txn {:#x} submitted.", txn_hash);

        if opt.blocking {
            ctx.state().watch_txn(txn_hash)?;
        }
        Ok(txn_hash)
    }
}
