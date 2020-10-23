// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::{bail, format_err, Result};
use scmd::{CommandAction, ExecContext};
use starcoin_resource_viewer::{AnnotatedMoveStruct, MoveValueAnnotator};
use starcoin_rpc_client::RemoteStateReader;
use starcoin_types::access_path::AccessPath;
use starcoin_vm_types::account_address::{parse_address, AccountAddress};
use starcoin_vm_types::account_config::account_struct_tag;
use starcoin_vm_types::language_storage::{StructTag, TypeTag};
use starcoin_vm_types::parser::parse_type_tag;
use structopt::StructOpt;

fn parse_struct_tag(s: &str) -> Result<StructTag> {
    let type_tag = parse_type_tag(s)?;
    match type_tag {
        TypeTag::Struct(st) => Ok(st),
        t => bail!("expect a struct tag, found: {:?}", t),
    }
}

//TODO support custom access_path.
#[derive(Debug, StructOpt)]
#[structopt(name = "get")]
pub struct GetOpt {
    #[structopt(short="a",long="addr", parse(try_from_str = parse_address))]
    /// address which the resource is under of. Default to default account address.
    account_address: Option<AccountAddress>,
    #[structopt(name = "struct-tag", parse(try_from_str = parse_struct_tag))]
    /// resource type to get. Default to 0x1::Account::Account
    struct_tag: Option<StructTag>,
}

pub struct GetCommand;

impl CommandAction for GetCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = GetOpt;
    type ReturnItem = AnnotatedMoveStruct;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        let opt = ctx.opt();
        let account_addr = match opt.account_address {
            Some(addr) => addr,
            None => ctx.state().default_account()?.address,
        };
        let struct_tag = match opt.struct_tag.as_ref() {
            Some(s) => s.clone(),
            None => account_struct_tag(),
        };
        let state = client
            .state_get(AccessPath::new(account_addr, struct_tag.access_vector()))?
            .ok_or_else(|| format_err!("Account with address {} state not exist.", account_addr))?;
        let account_state = client.state_get_account_state(account_addr).unwrap();
        dbg!(account_state);
        let chain_state_reader = RemoteStateReader::new(client);
        let viewer = MoveValueAnnotator::new(&chain_state_reader);
        let annotated_resource = viewer.view_struct(struct_tag, state.as_slice())?;

        Ok(annotated_resource)
    }
}
