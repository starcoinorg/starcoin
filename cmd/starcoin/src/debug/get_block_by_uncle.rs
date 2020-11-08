// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::view::BlockHeaderView;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use starcoin_crypto::HashValue;
use structopt::StructOpt;

///Query block by uncle hash
#[derive(Debug, StructOpt)]
#[structopt(name = "get_block_by_uncle")]
pub struct GetBlockByUncleOpt {
    ///Uncle hash
    #[structopt(short = "u")]
    uncle: HashValue,
}

pub struct GetBlockByUncleCommand;

impl CommandAction for GetBlockByUncleCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = GetBlockByUncleOpt;
    type ReturnItem = Option<BlockHeaderView>;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        let opt = ctx.opt();
        let block = client.chain_get_block_by_uncle(opt.uncle)?;

        match block {
            Some(b) => Ok(Some(b.into())),
            None => Ok(None),
        }
    }
}
