// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::{bail, Result};
use scmd::{CommandAction, ExecContext};
use short_hex_str::AsShortHexStr;
use starcoin_account_api::AccountPublicKey;
use starcoin_crypto::hash::PlainCryptoHash;
use starcoin_crypto::multi_ed25519::multi_shard::MultiEd25519SignatureShard;
use starcoin_dev::playground;
use starcoin_rpc_api::types::{FunctionIdView, TransactionOutputView, TransactionVMStatus};
use starcoin_rpc_client::RemoteStateReader;
use starcoin_state_api::AccountStateReader;
use starcoin_types::transaction::authenticator::TransactionAuthenticator;
use starcoin_types::transaction::{
    parse_transaction_argument, DryRunTransaction, RawUserTransaction, SignedUserTransaction,
    TransactionArgument,
};
use starcoin_vm_types::account_address::AccountAddress;
use starcoin_vm_types::token::stc::STC_TOKEN_CODE_STR;
use starcoin_vm_types::transaction::{ScriptFunction, TransactionPayload};
use starcoin_vm_types::transaction_argument::convert_txn_args;
use starcoin_vm_types::{language_storage::TypeTag, parser::parse_type_tag};
use std::env::current_dir;
use std::fs::File;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "sign-multisig-txn")]
/// Generate multisig txn running stdlib script or custom script.
/// And output the txn to file, waiting for other signers to sign the txn.
pub struct GenerateMultisigTxnOpt {
    #[structopt(name = "multisig-file")]
    /// mutlisig txn data generated by other participants.
    multisig_txn_file: Option<PathBuf>,

    #[structopt(short = "s", required_unless = "multisig-file")]
    /// sender address of this multisig txn.
    sender: Option<AccountAddress>,
    #[structopt(
        long = "function",
        name = "script-function",
        required_unless = "multisig-file"
    )]
    /// script function to execute, example: 0x1::TransferScripts::peer_to_peer
    script_function: Option<FunctionIdView>,

    #[structopt(
    short = "t",
    long = "type_tag",
    name = "type-tag",
    help = "can specify multi type_tag",
    parse(try_from_str = parse_type_tag)
    )]
    type_tags: Option<Vec<TypeTag>>,

    #[structopt(long = "arg", name = "transaction-arg",  parse(try_from_str = parse_transaction_argument))]
    /// transaction arguments
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
        let client = ctx.state().client();

        let type_tags = opt.type_tags.clone().unwrap_or_default();
        let args = opt.args.clone().unwrap_or_default();

        // gen multisig txn or read from file sent by other participants.
        let (signed_txn, existing_signatures) =
            if let Some(function_id) = opt.script_function.clone().map(|t| t.0) {
                let sender = ctx.opt().sender.expect("sender adress should be provided");
                let script_function = ScriptFunction::new(
                    function_id.module,
                    function_id.function,
                    type_tags,
                    convert_txn_args(&args),
                );
                let payload = TransactionPayload::ScriptFunction(script_function);

                let node_info = client.node_info()?;
                let chain_state_reader = RemoteStateReader::new(client)?;
                let account_state_reader = AccountStateReader::new(&chain_state_reader);
                let account_resource = account_state_reader.get_account_resource(&sender)?;

                if account_resource.is_none() {
                    bail!("address {} not exists on chain", &sender);
                }
                let account_resource = account_resource.unwrap();
                let expiration_time = opt.expiration_time + node_info.now_seconds;
                let raw_txn = RawUserTransaction::new(
                    sender,
                    account_resource.sequence_number(),
                    payload,
                    opt.max_gas_amount,
                    opt.gas_price,
                    expiration_time,
                    ctx.state().net().chain_id(),
                    STC_TOKEN_CODE_STR.to_string(),
                );
                (raw_txn, None)
            } else if let Some(file_input) = opt.multisig_txn_file.as_ref() {
                let txn: SignedUserTransaction =
                    bcs_ext::from_bytes(&std::fs::read(file_input.as_path())?)?;

                let existing_signatures = match txn.authenticator() {
                    TransactionAuthenticator::Ed25519 { .. } => {
                        bail!(
                            "expect a multisig txn in file {}",
                            file_input.as_path().display()
                        );
                    }
                    TransactionAuthenticator::MultiEd25519 {
                        public_key,
                        signature,
                    } => MultiEd25519SignatureShard::new(signature, *public_key.threshold()),
                };
                (txn.raw_txn().clone(), Some(existing_signatures))
            } else {
                unreachable!()
            };

        // sign the multi txn using my private keys.
        let sender = signed_txn.sender();
        let account = ctx
            .state()
            .client()
            .account_get(sender)?
            .ok_or_else(|| anyhow::anyhow!("cannot find multisig address {}", sender))?;
        let account_public_key = match &account.public_key {
            AccountPublicKey::Single(_) => {
                bail!("sender {} is not a multisig address", sender);
            }
            AccountPublicKey::Multi(m) => m.clone(),
        };

        // pre-run the txn.
        {
            let output: TransactionOutputView = {
                let state_view = RemoteStateReader::new(client)?;
                playground::dry_run(
                    &state_view,
                    DryRunTransaction {
                        public_key: AccountPublicKey::Multi(account_public_key.clone()),
                        raw_txn: signed_txn.clone(),
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
        }

        let partial_signed_txn = client.account_sign_txn(signed_txn)?;
        let my_signatures = if let TransactionAuthenticator::MultiEd25519 { signature, .. } =
            partial_signed_txn.authenticator()
        {
            MultiEd25519SignatureShard::new(signature, *account_public_key.threshold())
        } else {
            unreachable!()
        };

        // merge my signatures with existing signatures of other participants.
        let merged_signatures = {
            let mut signatures = vec![];
            if let Some(s) = existing_signatures {
                signatures.push(s);
            }
            signatures.push(my_signatures);
            MultiEd25519SignatureShard::merge(signatures)?
        };
        println!(
            "mutlisig txn(address: {}, threshold: {}): {} signatures collected",
            sender,
            merged_signatures.threshold(),
            merged_signatures.signatures().len()
        );
        if !merged_signatures.is_enough() {
            println!(
                "still require {} signatures",
                merged_signatures.threshold() as usize - merged_signatures.signatures().len()
            );
        } else {
            println!("enough signatures collected for the multisig txn, txn can be submitted now");
        }

        // construct the signed txn with merged signatures.
        let signed_txn = {
            let authenticator = TransactionAuthenticator::MultiEd25519 {
                public_key: account_public_key,
                signature: merged_signatures.into(),
            };
            SignedUserTransaction::new(partial_signed_txn.into_raw_transaction(), authenticator)
        };

        // output the txn, send this to other participants to sign, or just submit it.
        let output_file = {
            let mut output_dir = opt.output_dir.clone().unwrap_or(current_dir()?);
            // use hash's short str as output file name
            let file_name = signed_txn.crypto_hash().short_str();
            output_dir.push(file_name.as_str());
            output_dir.set_extension("multisig-txn");
            output_dir
        };
        let mut file = File::create(output_file.clone())?;
        // write txn to file
        bcs_ext::serialize_into(&mut file, &signed_txn)?;
        Ok(output_file)
    }
}
