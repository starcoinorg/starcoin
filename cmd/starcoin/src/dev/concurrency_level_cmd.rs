// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use clap::Parser;
use num_cpus;
use scmd::{CommandAction, ExecContext};
use starcoin_logger::prelude::*;

/// concurrency_level command option
#[derive(Debug, Parser)]
#[clap(name = "concurrency_level")]
pub struct ConcurrencyLevelCommandOpt {
    #[clap(name = "level", help = "set vm concurrency_level")]
    level: usize,
}

pub struct ConcurrencyLevelCommand;

impl CommandAction for ConcurrencyLevelCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = ConcurrencyLevelCommandOpt;
    type ReturnItem = String;

    fn run(&self, ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>) -> Result<String> {
        let opt = ctx.opt();
        let client = ctx.state().client();
        let concurrency_level = std::cmp::min(opt.level, num_cpus::get());
        client.set_concurrency_level(opt.level)?;
        info!("client set vm concurrency_level {}", concurrency_level);
        Ok(format!("set concurrency_level to {}", concurrency_level))
    }
}
