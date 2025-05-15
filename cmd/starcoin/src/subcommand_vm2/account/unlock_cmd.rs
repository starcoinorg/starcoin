// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use clap::Parser;
use scmd::{CommandAction, ExecContext};
use starcoin_vm2_account_api::AccountInfo;
use starcoin_vm2_types::account_address::AccountAddress;
use std::time::Duration;

/// Unlock the account
#[derive(Debug, Parser, Default)]
#[clap(name = "unlock")]
pub struct UnlockOpt {
    #[clap(short = 'p', default_value = "")]
    password: String,
    #[clap(
        short = 'd',
        help = "keep account unlock for how long(in seconds) from now",
        default_value = "300"
    )]
    duration: u32,
    #[clap(
        name = "account_address",
        help = "The wallet account address which to unlock, if absent, unlock the default wallet."
    )]
    account_address: Option<AccountAddress>,
}

pub struct UnlockCommand;

impl CommandAction for UnlockCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = UnlockOpt;
    type ReturnItem = AccountInfo;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let account_client = ctx.state().vm2()?.account_client();
        let opt: &UnlockOpt = ctx.opt();
        let account_address = if let Some(account_address) = opt.account_address {
            account_address
        } else {
            ctx.state().vm2()?.default_account()?.address
        };

        let duration = Duration::from_secs(opt.duration as u64);
        let account =
            account_client.unlock_account(account_address, opt.password.clone(), duration)?;
        Ok(account)
    }

    fn skip_history(&self, _ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>) -> bool {
        true
    }
}
