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
use starcoin_vm_types::parser;
use std::fs::OpenOptions;
use std::io::Read;
use std::time::Duration;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "execute")]
pub struct ExecuteOpt {
    #[structopt(name = "account_address", short = "a", long = "address")]
    account_address: AccountAddress,

    #[structopt(
        short = "f",
        name = "bytecode_file",
        help = "script bytecode file path"
    )]
    bytecode_file: String,
    #[structopt(
        short = "t",
        long = "type_tag",
        name = "type-tag",
        help = "can specify multi type_tag"
    )]
    type_tags: Vec<String>,
    #[structopt(long="arg", name="transaction-arg", help ="can specify multi arg", parse(try_from_str = parse_as_transaction_argument))]
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

        let mut type_tags = vec![];
        for type_tag in &opt.type_tags {
            type_tags.extend(parser::parse_type_tags(type_tag.as_ref())?.into_iter());
        }

        let args = opt.args.clone();

        let txn_address = opt.account_address;
        let client = ctx.state().client();
        let chain_state_reader = RemoteStateReader::new(client);
        let account_state_reader = AccountStateReader::new(&chain_state_reader);
        let account_resource = account_state_reader.get_account_resource(&txn_address)?;

        if account_resource.is_none() {
            bail!("address {} not exists on chain", &txn_address);
        }
        let account_resource = account_resource.unwrap();
        let expiration_time = Duration::from_secs(opt.expiration_time);
        let script_txn = RawUserTransaction::new_script(
            txn_address,
            account_resource.sequence_number(),
            Script::new(bytecode, type_tags, args),
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
            let block = client.watch_txn(txn_hash, Some(expiration_time * 2))?;
            println!(
                "txn mined in block hight: {}, hash: {:#x}",
                block.header().number(),
                block.header().id()
            );
        }
        Ok(txn_hash)
    }
}
