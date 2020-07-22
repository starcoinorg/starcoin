// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::view::BlockHeaderView;
use crate::StarcoinOpt;
use anyhow::{bail, Result};
use scmd::{CommandAction, ExecContext};
use starcoin_crypto::HashValue;
use starcoin_types::transaction::authenticator::AuthenticationKey;
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
    type ReturnItem = BlockHeaderView;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        unimplemented!()
    }
}
