// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::view::EventView;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use starcoin_crypto::HashValue;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "get_events")]
pub struct GetEventsOpt {
    #[structopt(name = "txn-info-id")]
    /// txn info id
    hash: HashValue,
}

pub struct GetEventsCommand;

impl CommandAction for GetEventsCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = GetEventsOpt;
    type ReturnItem = Vec<EventView>;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        let opt = ctx.opt();
        let events = client.chain_get_events_by_txn_info_id(opt.hash)?;
        let events = events.into_iter().map(|e| e.into()).collect::<Vec<_>>();
        Ok(events)
    }
}
