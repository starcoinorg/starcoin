// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use starcoin_logger::{prelude::*, LogPattern};
use structopt::StructOpt;

/// log level command option
#[derive(Debug, StructOpt)]
#[structopt(name = "level")]
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

/// Log pattern command option
#[derive(Debug, StructOpt)]
#[structopt(name = "pattern")]
pub struct LogPatternOpt {
    #[structopt(name = "pattern")]
    /// Set log pattern, support default|withline or custom pattern string.
    pattern: LogPattern,
}

pub struct LogPatternCommand;

impl CommandAction for LogPatternCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = LogPatternOpt;
    type ReturnItem = String;

    fn run(&self, ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>) -> Result<String> {
        let opt = ctx.opt();
        let client = ctx.state().client();
        client.debug_set_log_pattern(opt.pattern.clone())?;
        Ok(format!("set log pattern to {:?}", opt.pattern))
    }
}
