// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "new")]
pub struct NewOpt {
    #[structopt(short = "p")]
    password: String,
}

pub struct NewCommand {}

impl CommandAction for NewCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = NewOpt;

    fn run(&self, ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>) -> Result<()> {
        let client = ctx.state().client();
        println!("account new command, node: status: {:?}", client.status()?);
        Ok(())
    }
}
