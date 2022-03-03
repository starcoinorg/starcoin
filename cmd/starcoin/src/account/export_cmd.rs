// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::{bail, Result};
use scmd::{CommandAction, ExecContext};
use serde::{Deserialize, Serialize};
use starcoin_crypto::ValidCryptoMaterialStringExt;
use starcoin_types::transaction::authenticator::AccountPrivateKey;
use starcoin_vm_types::account_address::AccountAddress;
use std::convert::TryFrom;
use std::path::PathBuf;
use structopt::StructOpt;

/// Export account's private key.
#[derive(Debug, StructOpt)]
#[structopt(name = "export")]
pub struct ExportOpt {
    #[structopt(name = "account_address")]
    account_address: AccountAddress,
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
    type ReturnItem = ExportData;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().account_client();
        let opt: &ExportOpt = ctx.opt();
        let data = client.export_account(opt.account_address, opt.password.clone())?;
        let private_key = AccountPrivateKey::try_from(data.as_slice())?;
        let encoded = private_key.to_encoded_string()?;
        if let Some(output_file) = &opt.output_file {
            if output_file.exists() {
                bail!(
                    "the output_file {} is already exists, please change a name",
                    output_file.display()
                );
            }
            std::fs::write(output_file, encoded.clone())?;
            eprintln!("private key saved to {}", output_file.as_path().display());
        }
        Ok(ExportData {
            account: opt.account_address,
            private_key: encoded,
        })
    }

    fn skip_history(&self, _ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>) -> bool {
        true
    }
}

#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
pub struct ExportData {
    pub account: AccountAddress,
    pub private_key: String,
}
