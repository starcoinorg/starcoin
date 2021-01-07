// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use starcoin_vm_types::account_address::AccountAddress;
use structopt::StructOpt;

#[derive(Debug, StructOpt, Default)]
#[structopt(name = "change-password")]
pub struct ChangePasswordOpt {
    #[structopt(
        name = "account_address",
        help = "The wallet account address which to change password, if absent, use the default wallet."
    )]
    account_address: Option<AccountAddress>,

    #[structopt(short, name = "password")]
    password: String,
}

pub struct ChangePasswordCmd;

impl CommandAction for ChangePasswordCmd {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = ChangePasswordOpt;
    type ReturnItem = ();

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        let opt: &ChangePasswordOpt = ctx.opt();
        let account = ctx.state().get_account_or_default(opt.account_address)?;
        client.account_change_password(account.address, opt.password.clone())?;
        Ok(())
    }
}
