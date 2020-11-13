// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::{format_err, Result};
use scmd::{CommandAction, ExecContext};
use starcoin_types::peer_info::{Multiaddr, PeerId};
use structopt::StructOpt;

#[derive(Debug, StructOpt, Default)]
#[structopt(name = "get_address")]
///Get address by peer id
pub struct GetAddressOpt {
    #[structopt(name = "peer-id")]
    peer_id: Option<PeerId>,
}

pub struct GetAddressCommand;

impl CommandAction for GetAddressCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = GetAddressOpt;
    type ReturnItem = Vec<Multiaddr>;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        client.network_get_address(
            ctx.opt()
                .peer_id
                .clone()
                .ok_or_else(|| format_err!("Please input peer-id arg."))?
                .to_string(),
        )
    }
}
