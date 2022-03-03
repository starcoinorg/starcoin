// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::{bail, Result};
use scmd::{CommandAction, ExecContext};
use starcoin_account_api::{AccountInfo, AccountPrivateKey};
use starcoin_crypto::{ValidCryptoMaterial, ValidCryptoMaterialStringExt};
use starcoin_vm_types::account_address::AccountAddress;
use std::path::PathBuf;
use structopt::StructOpt;

/// Import account by private key to node wallet.
#[derive(Debug, StructOpt)]
#[structopt(name = "import")]
pub struct ImportOpt {
    #[structopt(short = "p", default_value = "")]
    password: String,

    #[structopt(name = "input", short = "i", help = "input of private key")]
    from_input: Option<String>,

    #[structopt(
        short = "f",
        help = "file path of private key",
        parse(from_os_str),
        conflicts_with("input")
    )]
    from_file: Option<PathBuf>,

    /// if account_address is absent, generate address by public_key.
    #[structopt(name = "account_address")]
    account_address: Option<AccountAddress>,
}

pub struct ImportCommand;

impl CommandAction for ImportCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = ImportOpt;
    type ReturnItem = AccountInfo;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let opt: &ImportOpt = ctx.opt();
        let client = ctx.state().account_client();
        let private_key = match (opt.from_input.as_ref(), opt.from_file.as_ref()) {
            (Some(p), _) => AccountPrivateKey::from_encoded_string(p)?,
            (None, Some(p)) => {
                let data = std::fs::read_to_string(p)?;
                AccountPrivateKey::from_encoded_string(data.as_str())?
            }
            (None, None) => {
                bail!("private key should be specified, use one of <input>, <from-file>")
            }
        };

        let address = opt
            .account_address
            .unwrap_or_else(|| private_key.public_key().derived_address());
        let account = client.import_account(
            address,
            private_key.to_bytes().to_vec(),
            opt.password.clone(),
        )?;
        Ok(account)
    }

    fn skip_history(&self, _ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>) -> bool {
        true
    }
}
