// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::mutlisig_transaction::MultisigTransaction;
use crate::StarcoinOpt;
use anyhow::{bail, Result};
use scmd::{CommandAction, ExecContext};
use starcoin_crypto::ed25519::Ed25519PublicKey;
use starcoin_crypto::hash::PlainCryptoHash;
use starcoin_crypto::multi_ed25519::MultiEd25519PublicKey;
use starcoin_crypto::ValidCryptoMaterialStringExt;
use starcoin_rpc_client::RemoteStateReader;
use starcoin_state_api::AccountStateReader;
use starcoin_transaction_builder::StdlibScript;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::transaction;
use starcoin_types::transaction::{
    parse_transaction_argument, RawUserTransaction, Script, TransactionArgument,
};
use starcoin_vm_types::transaction::helpers::get_current_timestamp;
use starcoin_vm_types::{language_storage::TypeTag, parser::parse_type_tag};
use std::env::current_dir;
use std::fs::{File, OpenOptions};
use std::io::Read;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "gen-multisig-txn")]
/// Generate multisig txn running stdlib script or custom script.
/// And output the txn to file, waiting for other signers to sign the txn.
pub struct GenerateMultisigTxnOpt {
    #[structopt(name = "sender", short = "s")]
    /// account address of the multisig account.
    sender: Option<AccountAddress>,

    #[structopt(short = "p", required = true, min_values = 1, max_values = 32, parse(try_from_str = Ed25519PublicKey::from_encoded_string))]
    /// public keys of the mutli-sig account.
    public_key: Vec<Ed25519PublicKey>,

    #[structopt(long)]
    /// the threshold of the mulisig account.
    threshold: Option<u8>,

    #[structopt(name = "stdlib-script", long = "stdlib-script")]
    /// stdlib script name, conflict with script-file option.
    stdlib_script: Option<StdlibScript>,

    #[structopt(
        name = "script-file",
        long = "script-file",
        conflicts_with = "stdlib-script"
    )]
    /// script bytecode file path
    script_file: Option<String>,

    #[structopt(
    short = "t",
    long = "type_tag",
    name = "type-tag",
    help = "can specify multi type_tag",
    parse(try_from_str = parse_type_tag)
    )]
    type_tags: Vec<TypeTag>,

    #[structopt(name = "transaction-arg",  parse(try_from_str = parse_transaction_argument))]
    /// transaction arguments
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
        long = "gas-price",
        name = "price of gas",
        default_value = "1",
        help = "gas price used to execute the script"
    )]
    gas_price: u64,
    #[structopt(name = "output-dir", long = "output-dir")]
    /// dir used to save raw txn data file. Default to current dir.
    output_dir: Option<PathBuf>,
}

pub struct GenerateMultisigTxnCommand;

impl CommandAction for GenerateMultisigTxnCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = GenerateMultisigTxnOpt;
    type ReturnItem = PathBuf;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let opt = ctx.opt();

        let threshold = opt.threshold.unwrap_or(opt.public_key.len() as u8);

        let multi_public_key = {
            // sort the public key to make account address derivation stable.
            let mut pubkeys = opt.public_key.clone();
            pubkeys.sort_by_key(|k| k.to_bytes());

            MultiEd25519PublicKey::new(pubkeys, threshold)?
        };

        let sender = if let Some(sender) = ctx.opt().sender {
            sender
        } else {
            let auth_key =
                transaction::authenticator::AuthenticationKey::multi_ed25519(&multi_public_key);
            auth_key.derived_address()
        };

        let bytecode = match (ctx.opt().stdlib_script, ctx.opt().script_file.clone()) {
            (Some(s), None) => s.compiled_bytes().into_vec(),
            (None, Some(bytecode_path)) => {
                let mut file = OpenOptions::new()
                    .read(true)
                    .write(false)
                    .open(bytecode_path)?;
                let mut bytecode = vec![];
                file.read_to_end(&mut bytecode)?;
                let _compiled_script =
                    match starcoin_vm_types::file_format::CompiledScript::deserialize(
                        bytecode.as_slice(),
                    ) {
                        Err(e) => {
                            bail!("invalid bytecode file, cannot deserialize as script, {}", e);
                        }
                        Ok(s) => s,
                    };
                bytecode
            }
            (None, None) => {
                bail!("either script-file or stdlib-script name should be provided");
            }
            (Some(_), Some(_)) => unreachable!(),
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
        let expiration_time = opt.expiration_time + get_current_timestamp();
        let script_txn = RawUserTransaction::new_script(
            sender,
            account_resource.sequence_number(),
            Script::new(bytecode, opt.type_tags.clone(), args),
            opt.max_gas_amount,
            opt.gas_price,
            expiration_time,
            ctx.state().net().chain_id(),
        );
        let txn = MultisigTransaction::new(
            script_txn.clone(),
            multi_public_key.public_keys().clone(),
            *multi_public_key.threshold(),
        );

        let output_file = {
            let mut output_dir = opt.output_dir.clone().unwrap_or(current_dir()?);
            // use hash's short str as output file name
            let file_name = script_txn.crypto_hash().short_str();
            output_dir.push(file_name);
            output_dir.set_extension("multisig-txn");
            output_dir
        };
        let mut file = File::create(output_file.clone())?;
        // write txn to file
        scs::serialize_into(&mut file, &txn)?;
        Ok(output_file)
    }
}
