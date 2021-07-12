// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use starcoin_rpc_api::types::StateWithProofView;
use starcoin_types::access_path::AccessPath;
use structopt::StructOpt;

/// Get state and proof with access_path, etc: 0x1/0/Account,  0x1/1/0x1::Account::Account
#[derive(Debug, StructOpt)]
#[structopt(name = "get-proof", alias = "get_proof")]
pub struct GetOpt {
    #[structopt(name = "access_path")]
    /// access_path of code or resource, etc: 0x1/0/Account,  0x1/1/0x1::Account::Account
    access_path: AccessPath,
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
        let proof = client.state_get_with_proof(opt.access_path.clone())?;
        Ok(proof)
    }
}
