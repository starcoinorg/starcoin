// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use starcoin_crypto::HashValue;
use starcoin_rpc_api::types::BlockHeaderView;
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
        _ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        //TODO filter block at here
        unimplemented!()
    }
}
