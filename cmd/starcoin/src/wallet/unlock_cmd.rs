// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use starcoin_types::account_address::AccountAddress;
use std::time::Duration;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
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
    #[structopt(name = "account_address")]
    account_address: AccountAddress,
}

pub struct UnlockCommand;

impl CommandAction for UnlockCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = UnlockOpt;
    type ReturnItem = String;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        let opt: &UnlockOpt = ctx.opt();
        let duration = Duration::from_secs(opt.duration as u64);
        client.wallet_unlock(opt.account_address.clone(), opt.password.clone(), duration)?;
        Ok(format!(
            "account {} unlocked for {:?}",
            &opt.account_address, duration
        ))
    }
}
