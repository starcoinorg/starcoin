// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "gen_block")]
pub struct GenBlockOpt {}

pub struct GenBlockCommand;

impl CommandAction for GenBlockCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = GenBlockOpt;
    type ReturnItem = ();

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        let net = ctx.state().net();
        assert!(net.is_dev());
        client.gen_block()
    }
}
