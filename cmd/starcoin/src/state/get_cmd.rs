// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::{bail, format_err, Result};
use scmd::{CommandAction, ExecContext};
use starcoin_resource_viewer::MoveValueAnnotator;
use starcoin_rpc_client::RemoteStateReader;
use starcoin_types::access_path::AccessPath;
use starcoin_vm_types::account_address::{parse_address, AccountAddress};
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
    #[structopt(name = "account_address", parse(try_from_str = parse_address))]
    account_address: AccountAddress,
    #[structopt(name = "struct-tag", parse(try_from_str = parse_struct_tag))]
    struct_tag: StructTag,
}

pub struct GetCommand;

impl CommandAction for GetCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = GetOpt;
    type ReturnItem = String;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        let opt = ctx.opt();
        let state = client
            .state_get(AccessPath::new(
                opt.account_address,
                opt.struct_tag.access_vector(),
            ))?
            .ok_or_else(|| {
                format_err!(
                    "Account with address {} state not exist.",
                    opt.account_address
                )
            })?;
        let chain_state_reader = RemoteStateReader::new(client);
        let viewer = MoveValueAnnotator::new(&chain_state_reader);
        let annotated_resource = viewer.view_struct(opt.struct_tag.clone(), state.as_slice())?;

        Ok(annotated_resource.to_string())
    }
}
