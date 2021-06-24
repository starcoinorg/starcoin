// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::{bail, Result};
use scmd::{CommandAction, ExecContext};
use structopt::StructOpt;

/// Let time pass for a period, only available in test or dev chain.
#[derive(Debug, StructOpt)]
#[structopt(name = "sleep")]
pub struct SleepOpt {
    #[structopt(
        short = "t",
        long = "time",
        name = "sleep time in milliseconds",
        default_value = "1",
        help = "only dev net can use."
    )]
    time: u64,
}

pub struct SleepCommand;

impl CommandAction for SleepCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = SleepOpt;
    type ReturnItem = ();

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let opt = ctx.opt();
        let client = ctx.state().client();
        let net = ctx.state().net();
        if !net.is_test_or_dev() {
            bail!(
                "This command only available in test or dev network, current network is: {}",
                net
            );
        }
        client.sleep(opt.time)
    }
}
