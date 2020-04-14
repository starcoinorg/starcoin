// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use starcoin_logger::prelude::*;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "loglevel")]
pub struct LogLevelOpt {
    #[structopt(short = "l")]
    level: Level,
}

pub struct LogLevelCommand;

impl CommandAction for LogLevelCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = LogLevelOpt;
    type ReturnItem = String;

    fn run(&self, ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>) -> Result<String> {
        let opt = ctx.opt();
        let client = ctx.state().client();
        client.debug_set_log_level(opt.level.clone())?;
        Ok(format!("set log level to {:?}", opt.level))
    }
}
