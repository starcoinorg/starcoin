use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::{bail, Result};
use scmd::{CommandAction, ExecContext};
use starcoin_crypto::hash::{HashValue, PlainCryptoHash};
use starcoin_rpc_client::RemoteStateReader;
use starcoin_state_api::AccountStateReader;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::account_config;
use starcoin_types::transaction::{Module, RawUserTransaction};
use starcoin_vm_types::{access::ModuleAccess, file_format::CompiledModule};
use std::fs::OpenOptions;
use std::io::Read;
use std::time::Duration;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "deploy")]
pub struct DeployOpt {
    #[structopt(
        short = "f",
        name = "bytecode_file",
        help = "module bytecode file path"
    )]
    bytecode_file: String,
    #[structopt(
        short = "g",
        name = "max-gas-amount",
        help = "max gas used to deploy the module"
    )]
    max_gas_amount: u64,
    #[structopt(
        name = "expiration_time",
        long = "timeout",
        default_value = "3000",
        help = "how long(in seconds) the txn stay alive"
    )]
    expiration_time: u64,
    #[structopt(
        short = "b",
        name = "blocking-mode",
        long = "blocking",
        help = "blocking wait txn mined"
    )]
    blocking: bool,
}

pub struct DeployCommand;

impl CommandAction for DeployCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = DeployOpt;
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
        let compiled_module = match CompiledModule::deserialize(bytecode.as_slice()) {
            Err(e) => {
                bail!("invalid bytecode file, cannot deserialize as module, {}", e);
            }
            Ok(compiled_module) => compiled_module,
        };
        let module_address = compiled_module.address().clone();
        // from libra address to our address
        let module_address = AccountAddress::new(module_address.into());
        let client = ctx.state().client();
        let chain_state_reader = RemoteStateReader::new(client);
        let account_state_reader = AccountStateReader::new(&chain_state_reader);
        let account_resource = account_state_reader.get_account_resource(&module_address)?;

        if account_resource.is_none() {
            bail!(
                "account of module address {} not exists on chain",
                &module_address
            );
        }

        let account_resource = account_resource.unwrap();

        let expiration_time = Duration::from_secs(opt.expiration_time);
        let deploy_txn = RawUserTransaction::new_module(
            module_address,
            account_resource.sequence_number(),
            Module::new(bytecode),
            opt.max_gas_amount,
            1,
            account_config::starcoin_type_tag(),
            expiration_time,
        );

        let signed_txn = client.wallet_sign_txn(deploy_txn)?;
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
