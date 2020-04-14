// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::state::CliState;
use crate::StarcoinOpt;
use anyhow::{bail, Result};
use scmd::{CommandAction, ExecContext};
use starcoin_crypto::{ed25519, ValidKeyStringExt};
use starcoin_types::account_address::AccountAddress;
use std::convert::TryFrom;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "export")]
pub struct ExportOpt {
    #[structopt(short = "a")]
    account: AccountAddress,
    #[structopt(short = "p", default_value = "")]
    password: String,
    #[structopt(short = "o", parse(from_os_str))]
    output_file: Option<PathBuf>,
}

pub struct ExportCommand;

impl CommandAction for ExportCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = ExportOpt;

    fn run(&self, ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>) -> Result<()> {
        let client = ctx.state().client();
        let opt: &ExportOpt = ctx.opt();
        let data = client.wallet_export(opt.account, opt.password.clone())?;
        let private_key = ed25519::Ed25519PrivateKey::try_from(data.as_slice())?;
        let encoded = private_key.to_encoded_string()?;
        if let Some(output_file) = &opt.output_file {
            if output_file.exists() {
                bail!("the output_file {} is already exists, please change a name");
            }
            std::fs::write(output_file, encoded.clone())?;
            println!("private key saved to {}", output_file.as_path().display());
        }
        println!("account {}, private key: {}", &opt.account, &encoded);
        Ok(())
    }
}
