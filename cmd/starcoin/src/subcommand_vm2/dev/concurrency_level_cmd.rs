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
#[clap(name = "set-concurrency-level", alias = "set_concurrency_level")]
pub struct SetConcurrencyLevelCommandOpt {
    #[clap(name = "level", help = "set vm concurrency_level")]
    level: usize,
}

pub struct SetConcurrencyLevelCommand;

impl CommandAction for SetConcurrencyLevelCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = SetConcurrencyLevelCommandOpt;
    type ReturnItem = String;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let opt = ctx.opt();
        let client = ctx.state().client();
        let concurrency_level = std::cmp::min(opt.level, num_cpus::get());
        client.set_concurrency_level(concurrency_level)?;
        info!("client set vm concurrency_level {}", concurrency_level);
        Ok(format!("set concurrency_level to {}", concurrency_level))
    }
}

/// get_concurrency_level command option
#[derive(Debug, Parser)]
#[clap(name = "get-concurrency-level", alias = "get_concurrency_level")]
pub struct GetConcurrencyLevelCommandOpt;

pub struct GetConcurrencyLevelCommand;

impl CommandAction for GetConcurrencyLevelCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = GetConcurrencyLevelCommandOpt;
    type ReturnItem = String;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().vm2()?.client();
        let level = client.get_concurrency_level()?;
        info!("client get vm concurrency_level {}", level);
        Ok(format!("get concurrency_level is {}", level))
    }
}
