// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use starcoin_account_api::AccountInfo;
use structopt::StructOpt;

/// Create a new account
#[derive(Debug, StructOpt, Default)]
#[structopt(name = "create")]
pub struct CreateOpt {
    #[structopt(short = "p")]
    password: String,
}

pub struct CreateCommand;

impl CommandAction for CreateCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = CreateOpt;
    type ReturnItem = AccountInfo;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<AccountInfo> {
        let client = ctx.state().client();
        let account = client.account_create(ctx.opt().password.clone())?;
        Ok(account)
    }

    fn skip_history(&self, _ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>) -> bool {
        true
    }
}
