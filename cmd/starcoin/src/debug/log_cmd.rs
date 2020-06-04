// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use starcoin_logger::prelude::*;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "loglevel")]
pub struct LogLevelOpt {
    #[structopt(name = "level")]
    level: Level,

    #[structopt(
        name = "logger",
        help = "set logger's level, if not present, set global level"
    )]
    logger_name: Option<String>,
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
        client.debug_set_log_level(opt.logger_name.clone(), opt.level)?;
        Ok(format!(
            "set {} log level to {:?}",
            opt.logger_name.as_deref().unwrap_or("global"),
            opt.level
        ))
    }
}
