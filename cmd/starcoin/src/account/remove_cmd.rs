// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use clap::Parser;
use scmd::{CommandAction, ExecContext};
use starcoin_account_api::AccountInfo;
use starcoin_vm_types::account_address::AccountAddress;

/// Remove account from local wallet. This operate do not affect the on chain account.
#[derive(Debug, Parser)]
#[clap(name = "remove")]
pub struct RemoveOpt {
    ///The account password, if the account is readonly account, do not require password
    #[clap(short = 'p')]
    password: Option<String>,
    #[clap(
        name = "account_address",
        help = "The wallet account address which to remove, the default account can not been removed."
    )]
    account_address: AccountAddress,
}

pub struct RemoveCommand;

impl CommandAction for RemoveCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = RemoveOpt;
    type ReturnItem = AccountInfo;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().account_client();
        let opt: &RemoveOpt = ctx.opt();
        let account_info = client.remove_account(opt.account_address, opt.password.clone())?;
        Ok(account_info)
    }

    fn skip_history(&self, _ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>) -> bool {
        true
    }
}
