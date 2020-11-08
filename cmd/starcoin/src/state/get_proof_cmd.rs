// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::view::StateWithProofView;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use starcoin_types::access_path::AccessPath;
use starcoin_vm_types::account_address::AccountAddress;
use starcoin_vm_types::account_config::AccountResource;
use starcoin_vm_types::move_resource::MoveResource;
use structopt::StructOpt;

//TODO support custom access_path.
#[derive(Debug, StructOpt)]
#[structopt(name = "get_proof")]
pub struct GetOpt {
    #[structopt(name = "account_address")]
    account_address: AccountAddress,
}

pub struct GetProofCommand;

impl CommandAction for GetProofCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = GetOpt;
    type ReturnItem = StateWithProofView;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        let opt = ctx.opt();
        let proof = client.state_get_with_proof(AccessPath::new(
            opt.account_address,
            AccountResource::resource_path(),
        ))?;

        Ok(proof.into())
    }
}
