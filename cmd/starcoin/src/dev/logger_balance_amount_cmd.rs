// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use clap::Parser;
use scmd::{CommandAction, ExecContext};
use starcoin_logger::prelude::*;

/// logger balance amount command option
#[derive(Debug, Parser)]
#[clap(name = "set_logger_balance_amount")]
pub struct SetLoggerBalanceAmountCommandOpt {
    #[clap(name = "balance_amount", help = "set logger balance amount in STC")]
    balance_amount: u64,
}

pub struct SetLoggerBalanceAmoutCommand;

impl CommandAction for SetLoggerBalanceAmoutCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = SetLoggerBalanceAmountCommandOpt;
    type ReturnItem = String;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let opt = ctx.opt();
        let client = ctx.state().client();
        const STC_SCALE: u64 = 1_000_000_000;
        let balance_amount = opt.balance_amount * STC_SCALE;
        client.set_logger_balance_amount(balance_amount)?;
        Ok(format!(
            "set logger balance amount {} STC",
            opt.balance_amount
        ))
    }
}

/// get_logger_balance_amount command option
#[derive(Debug, Parser)]
#[clap(name = "get_logger_balance_amount")]
pub struct GetLoggerBalanceAmountCommandOpt;

pub struct GetLoggerBalanceAmountCommand;

impl CommandAction for GetLoggerBalanceAmountCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = GetLoggerBalanceAmountCommandOpt;
    type ReturnItem = String;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        let balance_amount = client.get_logger_balance_amount()?;
        info!("client get logger balance amount {}", balance_amount);
        Ok(format!("get logger balance amount is {}", balance_amount))
    }
}
