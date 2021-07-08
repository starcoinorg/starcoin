// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::{format_err, Result};
use scmd::{CommandAction, ExecContext};
use starcoin_crypto::HashValue;
use starcoin_rpc_api::types::{AnnotatedMoveStructView, StructTagView};
use starcoin_vm_types::account_address::AccountAddress;
use std::collections::BTreeMap;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "list-resource")]
pub struct ListResourceOpt {
    #[structopt(name = "address")]
    /// address which the resources is under of.
    account_address: AccountAddress,
    #[structopt(name = "state_root")]
    /// state_root of the block for listing resource
    state_root: Option<HashValue>,
}

pub struct ListResourceCmd;

impl CommandAction for ListResourceCmd {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = ListResourceOpt;
    type ReturnItem = BTreeMap<StructTagView, AnnotatedMoveStructView>;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        let opt = ctx.opt();
        let account_addr = opt.account_address;
        let state_root = opt.state_root;
        let state = client
            .state_get_account_state_set(account_addr, state_root)?
            .ok_or_else(|| format_err!("Account with address {} state not exist.", account_addr))?;

        Ok(state.resources)
    }
}
