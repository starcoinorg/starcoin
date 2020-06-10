// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use starcoin_wallet_api::WalletAccount;
use structopt::StructOpt;

#[derive(Debug, StructOpt, Default)]
#[structopt(name = "list")]
pub struct ListOpt {}

pub struct ListCommand;

impl CommandAction for ListCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = ListOpt;
    type ReturnItem = Vec<WalletAccount>;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        let accounts = client.wallet_list()?;
        Ok(accounts)
    }
}
