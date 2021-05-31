// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::view::{ExecuteResultView, ExecutionOutputView};
use crate::StarcoinOpt;
use anyhow::{bail, Result};
use scmd::{CommandAction, ExecContext};
use starcoin_dev::playground;
use starcoin_move_compiler::load_bytecode_file;
use starcoin_rpc_api::types::{TransactionOutputView, TransactionVMStatus};
use starcoin_rpc_client::RemoteStateReader;
use starcoin_state_api::AccountStateReader;
use starcoin_types::transaction::{
    parse_transaction_argument, DryRunTransaction, RawUserTransaction, Script, TransactionArgument,
    TransactionPayload,
};
use starcoin_vm_types::account_address::AccountAddress;
use starcoin_vm_types::transaction_argument::convert_txn_args;
use starcoin_vm_types::{language_storage::TypeTag, parser::parse_type_tag};
use std::path::PathBuf;
use structopt::StructOpt;

/// Execute a script
#[derive(Debug, StructOpt)]
#[structopt(name = "execute-script")]
pub struct ExecuteScriptOpt {
    #[structopt(short = "s", long)]
    /// hex encoded string, like 0x1, 0x12
    sender: Option<AccountAddress>,

    #[structopt(
    short = "t",
    long = "type_tag",
    name = "type-tag",
    help = "can specify multi type_tag",
    parse(try_from_str = parse_type_tag)
    )]
    type_tags: Option<Vec<TypeTag>>,

    #[structopt(long = "arg", name = "transaction-args", help = "can specify multi arg", parse(try_from_str = parse_transaction_argument))]
    args: Option<Vec<TransactionArgument>>,

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
        default_value = "10000000",
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

    #[structopt(name = "mv_file", parse(from_os_str))]
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
        let client = ctx.state().client();
        let sender = if let Some(sender) = ctx.opt().sender {
            sender
        } else {
            ctx.state().default_account()?.address
        };
        let type_tags = opt.type_tags.clone().unwrap_or_default();
        let args = opt.args.clone().unwrap_or_default();

        let bytedata = {
            let move_file_path = opt.mv_file.clone();
            load_bytecode_file(move_file_path.as_path())?
        };

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

        let raw_txn = {
            let account_resource = {
                let chain_state_reader = RemoteStateReader::new(client)?;
                let account_state_reader = AccountStateReader::new(&chain_state_reader);
                account_state_reader.get_account_resource(&sender)?
            };

            if account_resource.is_none() {
                bail!("address {} not exists on chain", &sender);
            }
            let account_resource = account_resource.unwrap();

            let expiration_time = {
                let node_info = client.node_info()?;
                opt.expiration_time + node_info.now_seconds
            };
            RawUserTransaction::new_with_default_gas_token(
                sender,
                account_resource.sequence_number(),
                txn_payload,
                opt.max_gas_amount,
                opt.gas_price,
                expiration_time,
                ctx.state().net().chain_id(),
            )
        };

        let signed_txn = client.account_sign_txn(raw_txn)?;
        let txn_hash = signed_txn.id();
        let output: TransactionOutputView = {
            let state_view = RemoteStateReader::new(client)?;
            playground::dry_run(
                &state_view,
                DryRunTransaction {
                    public_key: signed_txn.authenticator().public_key(),
                    raw_txn: signed_txn.raw_txn().clone(),
                },
            )
            .map(|(_, b)| b.into())?
        };
        match output.status {
            TransactionVMStatus::Discard { status_code } => {
                bail!("TransactionStatus is discard: {:?}", status_code)
            }
            TransactionVMStatus::Executed => {}
            s => {
                bail!("pre-run failed, status: {:?}", s);
            }
        }
        if !opt.dry_run {
            client.submit_transaction(signed_txn)?;

            eprintln!("txn {:#x} submitted.", txn_hash);

            let mut output_view = ExecutionOutputView::new(txn_hash);

            if opt.blocking {
                let block = ctx.state().watch_txn(txn_hash)?.0;
                output_view.block_number = Some(block.header.number.0);
                output_view.block_id = Some(block.header.block_hash);
            }
            Ok(ExecuteResultView::Run(output_view))
        } else {
            Ok(ExecuteResultView::DryRun(output.into()))
        }
    }
}
