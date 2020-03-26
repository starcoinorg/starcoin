// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "loglevel")]
pub struct LogLevelOpt {
    #[structopt(short = "l")]
    level: String,
}

pub struct LogLevelCommand {}

impl CommandAction for LogLevelCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = LogLevelOpt;

    fn run(&self, ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>) -> Result<()> {
        let log_handler = ctx.state().logger();
        let opt = ctx.opt();
        log_handler.update_level(opt.level.as_str())?;
        println!(
            "set log level to {:?}, log file is {:?}",
            opt.level,
            log_handler.log_path()
        );
        Ok(())
    }
}
