// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;

use anyhow::{bail, Result};
use clap::{value_parser, Parser};
use starcoin_crypto::{ValidCryptoMaterial, ValidCryptoMaterialStringExt};

use scmd::{CommandAction, ExecContext};
use starcoin_account_api::AccountInfo;
use starcoin_rpc_api::types::TransactionStatusView;
use starcoin_types::identifier::Identifier;
use starcoin_types::language_storage::ModuleId;
use starcoin_vm_types::account_address::AccountAddress;
use starcoin_vm_types::account_config::core_code_address;
use starcoin_vm_types::transaction::authenticator::{AccountPrivateKey, AccountPublicKey};
use starcoin_vm_types::transaction::{EntryFunction, TransactionArgument, TransactionPayload};
use starcoin_vm_types::value::MoveValue;

use crate::cli_state::CliState;
use crate::view::TransactionOptions;
use crate::StarcoinOpt;

/// Rotate account's authentication key by specific private key. Return AccountInfo if Ok.
#[derive(Debug, Parser)]
#[clap(name = "rotate-authentication-key")]
pub struct RotateAuthKeyOpt {
    ///The account password
    #[clap(long = "password", default_value = "")]
    password: String,
    #[clap(
        name = "account_address",
        help = "The wallet account address which will be rotated, the default account can not be rotated."
    )]
    account_address: AccountAddress,

    #[command(flatten)]
    transaction_opts: TransactionOptions,

    #[clap(
        name = "input",
        short = 'i',
        help = "input of private key for rotating"
    )]
    from_input: Option<String>,
    #[arg(
        short = 'f',
        help = "file path of private key",
        value_parser = value_parser!(std::ffi::OsString),
        conflicts_with("input")
    )]
    from_file: Option<PathBuf>,
}

pub struct RotateAuthenticationKeyCommand;

impl CommandAction for RotateAuthenticationKeyCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = RotateAuthKeyOpt;
    type ReturnItem = AccountInfo;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().account_client();
        let opt: &RotateAuthKeyOpt = ctx.opt();

        let private_key = match (opt.from_input.as_ref(), opt.from_file.as_ref()) {
            (Some(p), _) => AccountPrivateKey::from_encoded_string(p)?,
            (None, Some(p)) => {
                let data = std::fs::read_to_string(p)?;
                AccountPrivateKey::from_encoded_string(data.as_str())?
            }
            (None, None) => {
                bail!("private key should be specified, use <input>, <from-file>")
            }
        };

        let account_public_key = match &private_key.public_key() {
            AccountPublicKey::Multi(_) => {
                bail!(
                    "{} is multisig address, you could use execute-function to rotate it step by step",
                    opt.account_address
                );
            }
            m => m.clone(),
        };

        let auth_key = account_public_key.authentication_key();
        let mut txn_opt = opt.transaction_opts.clone();
        txn_opt.blocking = true;
        txn_opt.sender = Option::from(opt.account_address);
        let result = ctx.state().build_and_execute_transaction(
            txn_opt,
            TransactionPayload::EntryFunction(EntryFunction::new(
                ModuleId::new(core_code_address(), Identifier::new("account").unwrap()),
                Identifier::new("rotate_authentication_key_call").unwrap(),
                vec![],
                vec![
                    MoveValue::from(TransactionArgument::U8Vector(auth_key.to_vec()))
                        .simple_serialize()
                        .expect("transaction arguments must serialize"),
                ],
            )),
        )?;

        if matches!(
            result.dry_run_output.txn_output.status,
            TransactionStatusView::Executed
        ) {
            client.remove_account(opt.account_address, Option::from(opt.password.clone()))?;

            let account_info = client.import_account(
                opt.account_address,
                private_key.to_bytes().to_vec(),
                opt.password.clone(),
            )?;
            return Ok(account_info);
        }

        bail!(
            "failed to execute rotate auth key script function: {:?}",
            result.dry_run_output.txn_output.status
        )
    }

    fn skip_history(&self, _ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>) -> bool {
        true
    }
}
