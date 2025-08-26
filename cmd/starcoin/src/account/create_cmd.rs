// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::CliState;
use anyhow::Result;
use clap::Parser;
use scmd::{CommandAction, ExecContext};
use starcoin_config::StarcoinOpt;
use starcoin_vm2_account_api::AccountInfo;

/// Create a new account
#[derive(Debug, Parser, Default)]
#[clap(name = "create")]
pub struct CreateOpt {
    #[clap(short = 'p')]
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
        let account_client = ctx.state().vm2()?.account_client();
        let account = account_client.create_account(ctx.opt().password.clone())?;
        Ok(account)
    }

    fn skip_history(&self, _ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>) -> bool {
        true
    }
}
