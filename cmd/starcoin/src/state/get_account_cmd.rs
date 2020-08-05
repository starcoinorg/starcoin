// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::view::AccountView;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use starcoin_rpc_client::RemoteStateReader;
use starcoin_state_api::AccountStateReader;
use starcoin_vm_types::account_address::{parse_address, AccountAddress};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "get_account")]
pub struct GetOpt {
    #[structopt(name = "account_address", parse(try_from_str = parse_address))]
    account_address: AccountAddress,
}

pub struct GetAccountCommand;

impl CommandAction for GetAccountCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = GetOpt;
    type ReturnItem = AccountView;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        let opt = ctx.opt();
        let chain_state_reader = RemoteStateReader::new(client);
        let account_state_reader = AccountStateReader::new(&chain_state_reader);
        let sequence_number = account_state_reader
            .get_account_resource(&opt.account_address)?
            .map(|res| res.sequence_number());
        let balance = account_state_reader.get_balance(&opt.account_address)?;

        Ok(AccountView {
            sequence_number,
            balance,
        })
    }
}
