// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use starcoin_account_api::{AccountInfo, AccountPublicKey};
use starcoin_crypto::{ValidCryptoMaterial, ValidCryptoMaterialStringExt};
use starcoin_vm_types::account_address::AccountAddress;
use structopt::StructOpt;

/// Import a readonly account by public key
#[derive(Debug, StructOpt)]
#[structopt(name = "import-readonly")]
pub struct ImportReadonlyOpt {
    #[structopt(name = "input", short = "i", help = "input of public key")]
    from_input: String,

    /// if account_address is absent, generate address by public_key.
    #[structopt(name = "account_address")]
    account_address: Option<AccountAddress>,
}

pub struct ImportReadonlyCommand;

impl CommandAction for ImportReadonlyCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = ImportReadonlyOpt;
    type ReturnItem = AccountInfo;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().account_client();
        let opt: &ImportReadonlyOpt = ctx.opt();

        let public_key = AccountPublicKey::from_encoded_string(opt.from_input.as_str())?;

        let address = opt
            .account_address
            .unwrap_or_else(|| public_key.derived_address());
        let account = client.import_readonly_account(address, public_key.to_bytes())?;
        Ok(account)
    }
}
