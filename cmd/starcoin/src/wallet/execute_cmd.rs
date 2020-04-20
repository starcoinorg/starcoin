use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::{bail, Result};
use scmd::{CommandAction, ExecContext};
use starcoin_crypto::hash::{CryptoHash, HashValue};
use starcoin_rpc_client::RemoteStateReader;
use starcoin_state_api::AccountStateReader;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::account_config;
use starcoin_types::language_storage::TypeTag;
use starcoin_types::transaction::{
    parse_as_transaction_argument, RawUserTransaction, Script, TransactionArgument,
};
use starcoin_vm_runtime::type_tag_parser::parse_type_tags;
use std::fs::OpenOptions;
use std::io::Read;
use std::time::Duration;
use structopt::StructOpt;
use vm as move_vm;

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
    #[structopt(short = "t", name = "type_tags", help = "type tags")]
    type_tags: Option<String>,
    #[structopt(long="args", name="transaction-args", parse(try_from_str = parse_as_transaction_argument))]
    args: Vec<TransactionArgument>,
    #[structopt(
        short = "g",
        name = "max-gas-amount",
        help = "max gas used to deploy the module"
    )]
    max_gas_amount: u64,
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
        let _compiled_script =
            match move_vm::file_format::CompiledScript::deserialize(bytecode.as_slice()) {
                Err(e) => {
                    bail!("invalid bytecode file, cannot deserialize as script, {}", e);
                }
                Ok(s) => s,
            };

        let type_tags = match opt.type_tags.as_ref() {
            None => vec![],
            Some(s) => parse_type_tags(s.as_str())?
                .into_iter()
                .map(|t| TypeTag::from(t))
                .collect(),
        };
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
        let script_txn = RawUserTransaction::new_script(
            txn_address,
            account_resource.sequence_number(),
            Script::new(bytecode, type_tags, args),
            opt.max_gas_amount,
            1,
            account_config::lbr_type_tag(),
            Duration::from_secs(40),
        );

        let signed_txn = client.wallet_sign_txn(script_txn)?;
        let txn_hash = CryptoHash::crypto_hash(&signed_txn);
        let succ = client.submit_transaction(signed_txn)?;
        if succ {
            Ok(txn_hash)
        } else {
            bail!("execute-txn is reject by node")
        }
    }
}
