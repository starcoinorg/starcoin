// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::{bail, Result};
use scmd::{CommandAction, ExecContext};
use starcoin_crypto::hash::{HashValue, PlainCryptoHash};
use starcoin_rpc_client::RemoteStateReader;
use starcoin_state_api::AccountStateReader;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::transaction::{
    parse_as_transaction_argument, RawUserTransaction, Script, TransactionArgument,
};
use starcoin_vm_types::{language_storage::TypeTag, parser::parse_type_tag};
use std::fs::OpenOptions;
use std::io::Read;
use std::time::Duration;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "execute")]
pub struct ExecuteOpt {
    #[structopt(name = "sender", short = "s", long = "sender")]
    sender: Option<AccountAddress>,

    #[structopt(
    short = "t",
    long = "type_tag",
    name = "type-tag",
    help = "can specify multi type_tag",
    parse(try_from_str = parse_type_tag)
    )]
    type_tags: Vec<TypeTag>,

    #[structopt(long = "arg", name = "transaction-args", help = "can specify multi arg", parse(try_from_str = parse_as_transaction_argument))]
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

    #[structopt(name = "bytecode_file", help = "script bytecode file path")]
    bytecode_file: String,
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
            ctx.state().default_account()?.address
        };

        let bytecode_path = ctx.opt().bytecode_file.clone();
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

        let args = opt.args.clone();

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
        if !succ {
            bail!("execute-txn is reject by node")
        }
        println!("txn {:#x} submitted.", txn_hash);

        if opt.blocking {
            ctx.state().watch_txn(txn_hash)?;
        }
        Ok(txn_hash)
    }
}
