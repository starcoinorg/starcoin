// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::{bail, Result};
use scmd::{CommandAction, ExecContext};
use starcoin_crypto::ed25519::Ed25519PrivateKey;
use starcoin_crypto::{PrivateKey, ValidCryptoMaterialStringExt};
use starcoin_types::account_address::{self, AccountAddress};
use starcoin_wallet_api::WalletAccount;
use std::path::PathBuf;
use structopt::StructOpt;

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
    type ReturnItem = WalletAccount;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        let opt: &ImportOpt = ctx.opt();

        let private_key = match (opt.from_input.as_ref(), opt.from_file.as_ref()) {
            (Some(p), _) => Ed25519PrivateKey::from_encoded_string(p)?,
            (None, Some(p)) => {
                let data = std::fs::read_to_string(p)?;
                Ed25519PrivateKey::from_encoded_string(data.as_str())?
            }
            (None, None) => {
                bail!("private key should be specified, use one of <input>, <from-file>")
            }
        };

        let address = opt
            .account_address
            .unwrap_or_else(|| account_address::from_public_key(&private_key.public_key()));
        let account = client.wallet_import(
            address,
            private_key.to_bytes().to_vec(),
            opt.password.clone(),
        )?;
        Ok(account)
    }
}
