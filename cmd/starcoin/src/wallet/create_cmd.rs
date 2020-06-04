// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use starcoin_wallet_api::WalletAccount;
use structopt::StructOpt;

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
    type ReturnItem = WalletAccount;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<WalletAccount> {
        let client = ctx.state().client();
        let account = client.wallet_create(ctx.opt().password.clone())?;
        Ok(account)
    }
}
