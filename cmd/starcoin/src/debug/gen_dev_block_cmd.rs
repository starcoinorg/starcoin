// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use starcoin_crypto::HashValue;
use starcoin_types::transaction::authenticator::AuthenticationKey;
use structopt::StructOpt;

///Generate block with dev consensus
#[derive(Debug, StructOpt)]
#[structopt(name = "gen_dev_block")]
pub struct GenDevBlockOpt {
    ///Parent hash
    #[structopt(short = "p")]
    parent: Option<HashValue>,
    ///Become master head
    #[structopt(short = "h")]
    head: bool,
}

pub struct GenDevBlockCommand;

impl CommandAction for GenDevBlockCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = GenDevBlockOpt;
    type ReturnItem = HashValue;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let net = ctx.state().net();
        net.assert_test_or_dev()?;

        let client = ctx.state().client();
        let opt = ctx.opt();
        let auth_key = AuthenticationKey::random();
        let new_block_id = client.create_dev_block(
            auth_key.derived_address(),
            auth_key.prefix().to_vec(),
            opt.parent,
            opt.head,
        )?;

        Ok(new_block_id)
    }
}
