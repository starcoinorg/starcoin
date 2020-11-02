// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::view::{ExecuteResultView, ExecutionOutputView};
use crate::StarcoinOpt;
use anyhow::{bail, format_err, Result};
use scmd::{CommandAction, ExecContext};
use starcoin_crypto::hash::PlainCryptoHash;
use starcoin_dev::playground;
use starcoin_move_compiler::{
    compile_source_string_no_report, errors, load_bytecode_file, CompiledUnit, MOVE_EXTENSION,
};
use starcoin_rpc_client::RemoteStateReader;
use starcoin_state_api::AccountStateReader;
use starcoin_transaction_builder::{compiled_transaction_script, StdlibScript};
use starcoin_types::transaction::{
    parse_transaction_argument, Module, RawUserTransaction, Script, TransactionArgument,
};
use starcoin_vm_types::account_address::{parse_address, AccountAddress};
use starcoin_vm_types::genesis_config::StdlibVersion;
use starcoin_vm_types::transaction::Transaction;
use starcoin_vm_types::vm_status::KeptVMStatus;
use starcoin_vm_types::{language_storage::TypeTag, parser::parse_type_tag};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "execute")]
pub struct ExecuteOpt {
    #[structopt(short = "s", long, parse(try_from_str = parse_address))]
    /// hex encoded string, like 0x1, 0x12
    sender: Option<AccountAddress>,

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
    #[structopt(long = "dry-run")]
    /// dry-run script, only get transaction output, no state change to chain
    dry_run: bool,

    #[structopt(long = "local")]
    /// Whether dry-run in local cli or remote node.
    local_mode: bool,

    #[structopt(long = "script", name = "builtin-script")]
    /// builtin script name to execute
    script_name: Option<StdlibScript>,

    #[structopt(
        name = "move_file",
        parse(from_os_str),
        required_unless = "builtin-script"
    )]
    /// bytecode file or move script source file
    move_file: Option<PathBuf>,

    #[structopt(name = "dependency_path", long = "dep")]
    /// path of dependency used to build, only used when using move source file
    deps: Vec<String>,
}

pub struct ExecuteCommand;

impl CommandAction for ExecuteCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = ExecuteOpt;
    type ReturnItem = ExecuteResultView;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let opt = ctx.opt();
        let sender = if let Some(sender) = ctx.opt().sender {
            sender
        } else {
            ctx.state().default_account()?.address
        };

        let (bytecode, is_script) = if let Some(builtin_script) = opt.script_name.as_ref() {
            let code =
                compiled_transaction_script(StdlibVersion::Latest, *builtin_script).into_vec();
            (code, true)
        } else {
            let move_file_path = ctx
                .opt()
                .move_file
                .clone()
                .ok_or_else(|| format_err!("expect a move file path"))?;
            let ext = move_file_path
                .as_path()
                .extension()
                .map(|os_str| os_str.to_str().expect("file extension should is utf8 str"))
                .unwrap_or_else(|| "");
            if ext == MOVE_EXTENSION {
                let mut deps = stdlib::stdlib_files();
                // add extra deps
                deps.append(&mut ctx.opt().deps.clone());
                let (sources, compile_result) = compile_source_string_no_report(
                    std::fs::read_to_string(move_file_path.as_path())?.as_str(),
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

                let is_script = match compile_unit {
                    CompiledUnit::Module { .. } => false,
                    CompiledUnit::Script { .. } => true,
                };
                (compile_unit.serialize(), is_script)
            } else {
                load_bytecode_file(move_file_path.as_path())?
            }
        };

        let type_tags = opt.type_tags.clone();
        let args = opt.args.clone();

        let client = ctx.state().client();
        let node_info = client.node_info()?;
        let chain_state_reader = RemoteStateReader::new(client);
        let account_state_reader = AccountStateReader::new(&chain_state_reader);
        let account_resource = account_state_reader.get_account_resource(&sender)?;

        if account_resource.is_none() {
            bail!("address {} not exists on chain", &sender);
        }
        let account_resource = account_resource.unwrap();

        let expiration_time = opt.expiration_time + node_info.now_seconds;
        let script_txn = if is_script {
            RawUserTransaction::new_script(
                sender,
                account_resource.sequence_number(),
                Script::new(bytecode, type_tags, args),
                opt.max_gas_amount,
                opt.gas_price,
                expiration_time,
                ctx.state().net().chain_id(),
            )
        } else {
            RawUserTransaction::new_module(
                sender,
                account_resource.sequence_number(),
                Module::new(bytecode),
                opt.max_gas_amount,
                opt.gas_price,
                expiration_time,
                ctx.state().net().chain_id(),
            )
        };

        let signed_txn = client.account_sign_txn(script_txn)?;
        let txn_hash = signed_txn.crypto_hash();
        let (vm_status, output) = if opt.local_mode {
            let state_view = RemoteStateReader::new(client);
            playground::dry_run(
                &state_view,
                Transaction::UserTransaction(signed_txn.clone()),
            )?
        } else {
            client.dry_run(signed_txn.clone())?
        };
        let keep_status = output.status().status().map_err(|status_code| {
            format_err!("TransactionStatus is discard: {:?}", status_code)
        })?;
        if keep_status != KeptVMStatus::Executed {
            bail!(
                "move file pre-run failed, {:?}, vm_status: {:?}",
                keep_status,
                vm_status
            );
        }
        if !opt.dry_run {
            let success = client.submit_transaction(signed_txn)?;
            if let Err(e) = success {
                bail!("execute-txn is reject by node, reason: {}", &e)
            }
            println!("txn {:#x} submitted.", txn_hash);

            let mut output_view = ExecutionOutputView::new(txn_hash);

            if opt.blocking {
                let block = ctx.state().watch_txn(txn_hash)?;
                output_view.block_number = Some(block.header().number);
                output_view.block_id = Some(block.header().id());
            }
            Ok(ExecuteResultView::Run(output_view))
        } else {
            Ok(ExecuteResultView::DryRun(output.into()))
        }
    }
}
