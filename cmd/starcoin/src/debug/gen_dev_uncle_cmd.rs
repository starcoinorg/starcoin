// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::{bail, Result};
use scmd::{CommandAction, ExecContext};
use starcoin_crypto::HashValue;
use starcoin_types::transaction::authenticator::AuthenticationKey;
use structopt::StructOpt;

///Generate Uncle block with dev consensus
#[derive(Debug, StructOpt)]
#[structopt(name = "gen_dev_uncle")]
pub struct GenDevUncleOpt {
    ///Parent hash
    #[structopt(short = "p")]
    parent: Option<HashValue>,
}

pub struct GenDevUncleCommand;

impl CommandAction for GenDevUncleCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = GenDevUncleOpt;
    type ReturnItem = HashValue;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let net = ctx.state().net();
        if !net.is_dev() {
            bail!("This command only work for dev network");
        }

        let client = ctx.state().client();
        let opt = ctx.opt();
        let auth_key = AuthenticationKey::random();
        let new_block_id = client.create_dev_block(
            auth_key.derived_address(),
            auth_key.prefix().to_vec(),
            opt.parent,
        )?;

        Ok(new_block_id)
    }
}
