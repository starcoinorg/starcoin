// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "list")]
pub struct ListOpt {}

pub struct ListCommand {}

impl CommandAction for ListCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = ListOpt;

    fn run(&self, ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>) -> Result<()> {
        let client = ctx.state().client();
        let accounts = client.account_list()?;
        for account in accounts {
            println!("{} {}", account.address, account.is_default);
        }
        Ok(())
    }
}
