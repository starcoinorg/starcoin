// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use structopt::StructOpt;

///Manual trigger panic, only work for dev network.
#[derive(Debug, StructOpt)]
#[structopt(name = "panic")]
pub struct PanicOpt {
    /// if set remote, panic at Backend, otherwise panic at cli.
    #[structopt(short = "r")]
    remote: bool,
}

pub struct PanicCommand;

impl CommandAction for PanicCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = PanicOpt;
    type ReturnItem = String;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let opt = ctx.opt();
        let client = ctx.state().client();
        let net = ctx.state().net();
        net.assert_test_or_dev()?;
        if opt.remote {
            client.debug_panic()?;
        } else {
            panic!("Debug command panic.");
        }
        Ok("Remote node is panic.".to_string())
    }
}
