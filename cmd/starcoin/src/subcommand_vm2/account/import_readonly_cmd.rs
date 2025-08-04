// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::CliState;
use anyhow::Result;
use clap::Parser;
use scmd::{CommandAction, ExecContext};
use starcoin_config::StarcoinOpt;
use starcoin_vm2_account_api::AccountInfo;
use starcoin_vm2_crypto::{ValidCryptoMaterial, ValidCryptoMaterialStringExt};
use starcoin_vm2_types::account_address::AccountAddress;
use starcoin_vm2_vm_types::transaction::authenticator::AccountPublicKey;

/// Import a readonly account by public key
#[derive(Debug, Parser)]
#[clap(name = "import-readonly")]
pub struct ImportReadonlyOpt {
    #[clap(name = "input", short = 'i', help = "input of public key")]
    from_input: String,

    /// if account_address is absent, generate address by public_key.
    #[clap(name = "account_address")]
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
        let client = ctx.state().vm2()?.account_client();
        let opt: &ImportReadonlyOpt = ctx.opt();

        let public_key = AccountPublicKey::from_encoded_string(opt.from_input.as_str())?;

        let address = opt
            .account_address
            .unwrap_or_else(|| public_key.derived_address());
        let account = client.import_readonly_account(address, public_key.to_bytes())?;
        Ok(account)
    }
}
