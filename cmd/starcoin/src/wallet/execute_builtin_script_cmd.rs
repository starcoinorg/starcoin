// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::{bail, Result};
use scmd::{CommandAction, ExecContext};
use starcoin_crypto::hash::{HashValue, PlainCryptoHash};
use starcoin_rpc_client::RemoteStateReader;
use starcoin_state_api::AccountStateReader;
use starcoin_transaction_builder::StdlibScript;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::transaction::{
    parse_transaction_argument, RawUserTransaction, Script, TransactionArgument,
};
use starcoin_vm_types::chain_config::ChainId;
use starcoin_vm_types::transaction::helpers::get_current_timestamp;
use starcoin_vm_types::{language_storage::TypeTag, parser::parse_type_tag};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "execute-builtin")]
pub struct ExecuteBuiltInScriptOpt {
    #[structopt(short = "s", long = "sender")]
    /// if `sender` is absent, use default account.
    sender: Option<AccountAddress>,

    #[structopt(long = "script", name = "script-name")]
    /// builtin script name to execute
    script_name: StdlibScript,

    #[structopt(
    short = "t",
    long = "type_tag",
    name = "type-tag",
    parse(try_from_str = parse_type_tag)
    )]
    /// type tags for the script
    type_tags: Vec<TypeTag>,

    #[structopt(name = "transaction-args", parse(try_from_str = parse_transaction_argument))]
    /// args for the script.
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
        name = "max-gas-amount",
        default_value = "1000000",
        help = "max gas used to deploy the module"
    )]
    max_gas_amount: u64,
    #[structopt(
        short = "p",
        long = "gas-price",
        name = "price of gas",
        default_value = "1",
        help = "gas price used to deploy the module"
    )]
    gas_price: u64,

    #[structopt(
        short = "b",
        name = "blocking-mode",
        long = "blocking",
        help = "blocking wait txn mined"
    )]
    blocking: bool,
}

pub struct ExecuteBuildInCommand;

impl CommandAction for ExecuteBuildInCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = ExecuteBuiltInScriptOpt;
    type ReturnItem = HashValue;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let opt = ctx.opt();
        let client = ctx.state().client();

        let sender = ctx.state().wallet_account_or_default(opt.sender)?;
        let chain_state_reader = RemoteStateReader::new(client);
        let account_state_reader = AccountStateReader::new(&chain_state_reader);
        let account_resource = account_state_reader.get_account_resource(&sender.address)?;

        if account_resource.is_none() {
            bail!(
                "account of module address {} not exists on chain",
                sender.address
            );
        }

        let account_resource = account_resource.unwrap();
        let expiration_time = opt.expiration_time + get_current_timestamp();

        let bytecode = opt.script_name.compiled_bytes().into_vec();
        let type_tags = opt.type_tags.clone();
        let args = opt.args.clone();

        let script_txn = RawUserTransaction::new_script(
            sender.address,
            account_resource.sequence_number(),
            Script::new(bytecode, type_tags, args),
            opt.max_gas_amount,
            opt.gas_price,
            expiration_time,
            ChainId::new(ctx.state().net().chain_id()),
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
