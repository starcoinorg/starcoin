// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{cli_state::CliState, StarcoinOpt};
use anyhow::Result;
use clap::Parser;
use scmd::{CommandAction, ExecContext};

///Manual trigger panic, only work for dev network.
#[derive(Debug, Parser)]
#[clap(name = "panic")]
pub struct PanicOpt {
    /// if set remote, panic at Backend, otherwise panic at cli.
    #[clap(short = 'r')]
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
        let client = ctx.state().vm2()?.client();
        let net = ctx.state().vm2()?.net();
        net.assert_test_or_dev()?;
        if opt.remote {
            client.debug_panic()?;
        } else {
            panic!("Debug command panic.");
        }
        Ok("Remote node is panic.".to_string())
    }
}
