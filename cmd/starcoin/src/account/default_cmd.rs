// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use starcoin_account_api::AccountInfo;
use starcoin_vm_types::account_address::AccountAddress;
use structopt::StructOpt;

/// Set or show the default account
#[derive(Debug, StructOpt, Default)]
#[structopt(name = "default")]
pub struct DefaultOpt {
    #[structopt(
        name = "account_address",
        help = "set default address to this, if not provided, display current default address"
    )]
    account_address: Option<AccountAddress>,
}

pub struct DefaultCommand;

impl CommandAction for DefaultCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = DefaultOpt;
    type ReturnItem = AccountInfo;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let opt: &DefaultOpt = ctx.opt();

        match opt.account_address.as_ref() {
            None => {
                let default_account = ctx.state().default_account()?;
                Ok(default_account)
            }
            Some(addr) => {
                let client = ctx.state().client();
                let default_account = client.set_default_account(*addr)?;
                Ok(default_account)
            }
        }
    }
}
