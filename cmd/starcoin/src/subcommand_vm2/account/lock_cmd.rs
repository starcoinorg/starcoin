// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::view::StringView;
use crate::StarcoinOpt;
use anyhow::Result;
use clap::Parser;
use scmd::{CommandAction, ExecContext};
use starcoin_vm2_vm_types::account_address::AccountAddress;

/// Lock the account
#[derive(Debug, Parser, Default)]
#[clap(name = "lock")]
pub struct LockOpt {
    #[clap(
        name = "account_address",
        help = "The wallet account address witch to lock, if absent, lock the default wallet."
    )]
    account_address: Option<AccountAddress>,
}

pub struct LockCommand;

impl CommandAction for LockCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = LockOpt;
    type ReturnItem = StringView;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().vm2()?.account_client();
        let opt: &LockOpt = ctx.opt();
        let account = ctx
            .state()
            .vm2()?
            .get_account_or_default(opt.account_address)?;

        client.lock_account(account.address)?;
        Ok(StringView {
            result: account.address.to_string(),
        })
    }
}
