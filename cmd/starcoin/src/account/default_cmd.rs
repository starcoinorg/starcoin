// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::view::StringView;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use starcoin_vm_types::account_address::{parse_address, AccountAddress};
use structopt::StructOpt;

#[derive(Debug, StructOpt, Default)]
#[structopt(name = "default")]
pub struct DefaultOpt {
    #[structopt(
        name = "account_address",
        help = "set default address to this, if not provided, display current default address",
        parse(try_from_str = parse_address),
    )]
    account_address: Option<AccountAddress>,
}

pub struct DefaultCommand;

impl CommandAction for DefaultCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = DefaultOpt;
    type ReturnItem = StringView;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let opt: &DefaultOpt = ctx.opt();

        match opt.account_address.as_ref() {
            None => {
                let default_account = ctx.state().default_account()?;
                Ok(StringView {
                    result: default_account.address.to_string(),
                })
            }
            Some(addr) => {
                let client = ctx.state().client();
                client.set_default_account(*addr)?;
                Ok(StringView {
                    result: addr.to_string(),
                })
            }
        }
    }
}
