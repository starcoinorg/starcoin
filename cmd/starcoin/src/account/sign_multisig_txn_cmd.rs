// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::convert::TryInto;
use std::env::current_dir;
use std::path::PathBuf;

use anyhow::{bail, Result};
use clap::Parser;

use scmd::{CommandAction, ExecContext};
use starcoin_account_api::AccountPublicKey;
use starcoin_rpc_api::types::{FunctionIdView, RawUserTransactionView, TransactionStatusView};
use starcoin_rpc_client::StateRootOption;
use starcoin_state_api::StateReaderExt;
use starcoin_types::transaction::{
    parse_transaction_argument_advance, DryRunTransaction, RawUserTransaction, TransactionArgument,
};
use starcoin_vm_types::account_address::AccountAddress;
use starcoin_vm_types::token::stc::STC_TOKEN_CODE_STR;
use starcoin_vm_types::transaction::{ScriptFunction, TransactionPayload};
use starcoin_vm_types::transaction_argument::convert_txn_args;
use starcoin_vm_types::{language_storage::TypeTag, parser::parse_type_tag};

use crate::cli_state::CliState;
use crate::mutlisig_transaction::read_multisig_existing_signatures;
use crate::StarcoinOpt;

#[derive(Debug, Parser)]
#[clap(name = "sign-multisig-txn")]
/// Generate multisig txn running stdlib script or custom script.
/// And output the txn to file, waiting for other signers to sign the txn.
pub struct GenerateMultisigTxnOpt {
    #[clap(name = "multisig-file")]
    /// mutlisig txn data generated by other participants.
    multisig_txn_file: Option<PathBuf>,

    #[clap(short = 's', required_unless_present = "multisig-file")]
    /// sender address of this multisig txn.
    sender: Option<AccountAddress>,
    #[clap(
        long = "function",
        name = "script-function",
        required_unless_present = "multisig-file"
    )]
    /// script function to execute, example: 0x1::TransferScripts::peer_to_peer_v2
    script_function: Option<FunctionIdView>,

    #[clap(
    short = 't',
    long = "type_tag",
    name = "type-tag",
    help = "can specify multi type_tag",
    parse(try_from_str = parse_type_tag)
    )]
    type_tags: Option<Vec<TypeTag>>,

    #[clap(long = "arg", name = "transaction-arg", parse(try_from_str = parse_transaction_argument_advance))]
    /// transaction arguments
    args: Option<Vec<TransactionArgument>>,

    #[clap(
        name = "expiration_time",
        long = "timeout",
        default_value = "3000",
        help = "how long(in seconds) the txn stay alive"
    )]
    expiration_time: u64,

    #[clap(
        short = 'g',
        long = "max-gas",
        name = "max-gas-amount",
        default_value = "10000000",
        help = "max gas used to execute the script"
    )]
    max_gas_amount: u64,
    #[clap(
        long = "gas-price",
        name = "price of gas",
        default_value = "1",
        help = "gas price used to execute the script"
    )]
    gas_price: u64,

    #[clap(name = "output-dir", long = "output-dir")]
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
        let rpc_client = ctx.state().client();
        let account_client = ctx.state().account_client();

        let type_tags = opt.type_tags.clone().unwrap_or_default();
        let args = opt.args.clone().unwrap_or_default();

        // gen multisig txn or read from file sent by other participants.
        let (raw_txn, existing_signatures) =
            if let Some(function_id) = opt.script_function.clone().map(|t| t.0) {
                let sender = ctx.opt().sender.expect("sender address should be provided");
                let script_function = ScriptFunction::new(
                    function_id.module,
                    function_id.function,
                    type_tags,
                    convert_txn_args(&args),
                );
                let payload = TransactionPayload::ScriptFunction(script_function);

                let node_info = rpc_client.node_info()?;
                let chain_state_reader = rpc_client.state_reader(StateRootOption::Latest)?;
                let account_resource = chain_state_reader.get_account_resource(sender)?;

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
                let read_ret = read_multisig_existing_signatures(file_input.as_path()).unwrap();
                (read_ret.txn, read_ret.signatures)
            } else {
                unreachable!()
            };
        let mut raw_txn_view: RawUserTransactionView = raw_txn.clone().try_into()?;
        raw_txn_view.decoded_payload = Some(
            ctx.state()
                .decode_txn_payload(raw_txn.payload())?
                .try_into()?,
        );
        // Use `eprintln` instead of `println`, for keep the cli stdout's format(such as json) is not broken by print.
        eprintln!(
            "Prepare to sign the transaction: \n {}",
            serde_json::to_string_pretty(&raw_txn_view)?
        );
        // sign the multi txn using my private keys.
        let sender = raw_txn.sender();
        let account = account_client
            .get_account(sender)?
            .ok_or_else(|| anyhow::anyhow!("cannot find multisig address {}", sender))?;
        let account_public_key = match &account.public_key {
            AccountPublicKey::Single(_) => {
                bail!("sender {} is not a multisig address", sender);
            }
            AccountPublicKey::Multi(m) => m.clone(),
        };
        // pre-run the txn when first generation.
        if opt.multisig_txn_file.is_none() {
            let output = ctx.state().client().dry_run_raw(DryRunTransaction {
                public_key: AccountPublicKey::Multi(account_public_key.clone()),
                raw_txn: raw_txn.clone(),
            })?;

            eprintln!(
                "Transaction dry run execute output: \n {}",
                serde_json::to_string_pretty(&output)?
            );
            match &output.txn_output.status {
                TransactionStatusView::Discard {
                    status_code,
                    status_code_name,
                } => {
                    bail!(
                        "TransactionStatus is discard: {:?}, {}",
                        status_code,
                        status_code_name
                    )
                }
                TransactionStatusView::Executed => {}
                s => {
                    bail!("pre-run failed, status: {:?}", s);
                }
            }
        }

        let mut output_dir = opt.output_dir.clone().unwrap_or(current_dir()?);
        let _ = ctx.state().sign_multisig_txn_to_file_or_submit(
            raw_txn.sender(),
            account_public_key,
            existing_signatures,
            account_client.sign_txn(raw_txn, sender)?,
            &mut output_dir,
            false,
            false,
        )?;

        Ok(output_dir)
    }
}
