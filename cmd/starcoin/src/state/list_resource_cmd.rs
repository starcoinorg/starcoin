// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::{format_err, Result};
use scmd::{CommandAction, ExecContext};
use starcoin_resource_viewer::{AnnotatedMoveStruct, MoveValueAnnotator};
use starcoin_rpc_api::types::StructTagView;
use starcoin_rpc_client::RemoteStateReader;
use starcoin_types::access_path::AccessPath;
use starcoin_vm_types::account_address::AccountAddress;
use starcoin_vm_types::account_config::account_struct_tag;
use starcoin_vm_types::language_storage::StructTag;
use starcoin_vm_types::parser::parse_struct_tag;
use std::collections::BTreeMap;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "list-resource")]
pub struct ListResourceOpt {
    #[structopt(name = "address")]
    /// address which the resources is under of.
    account_address: AccountAddress,
}

pub struct ListResourceCmd;

impl CommandAction for ListResourceCmd {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = ListResourceOpt;
    type ReturnItem = BTreeMap<StructTagView, AnnotatedMoveStruct>;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        let opt = ctx.opt();
        let account_addr = opt.account_address;

        let state = client
            .get_account_state_set(account_addr)?
            .ok_or_else(|| format_err!("Account with address {} state not exist.", account_addr))?;

        Ok(state.resources)
    }
}
