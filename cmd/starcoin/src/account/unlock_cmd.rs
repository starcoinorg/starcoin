// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::view::StringView;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use starcoin_vm_types::account_address::AccountAddress;
use std::time::Duration;
use structopt::StructOpt;

#[derive(Debug, StructOpt, Default)]
#[structopt(name = "unlock")]
pub struct UnlockOpt {
    #[structopt(short = "p", default_value = "")]
    password: String,
    #[structopt(
        short = "d",
        help = "keep account unlock for how long(in seconds) from now",
        default_value = "300"
    )]
    duration: u32,
    #[structopt(
        name = "account_address",
        help = "The wallet account address witch to unlock, if absent, unlock the default wallet."
    )]
    account_address: Option<AccountAddress>,
}

pub struct UnlockCommand;

impl CommandAction for UnlockCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = UnlockOpt;
    type ReturnItem = StringView;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        let opt: &UnlockOpt = ctx.opt();

        let account = ctx.state().get_account_or_default(opt.account_address)?;

        let duration = Duration::from_secs(opt.duration as u64);
        client.account_unlock(account.address, opt.password.clone(), duration)?;
        Ok(StringView {
            result: account.address.to_string(),
        })
    }
}
