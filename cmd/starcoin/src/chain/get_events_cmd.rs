// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use starcoin_crypto::HashValue;
use starcoin_rpc_api::chain::GetEventOption;
use starcoin_rpc_api::types::TransactionEventResponse;
use structopt::StructOpt;

/// Get chain's events by txn hash
#[derive(Debug, StructOpt)]
#[structopt(name = "get-events", alias = "get_events")]
pub struct GetEventsOpt {
    #[structopt(name = "txn-hash")]
    /// txn hash
    hash: HashValue,
}

pub struct GetEventsCommand;

impl CommandAction for GetEventsCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = GetEventsOpt;
    type ReturnItem = Vec<TransactionEventResponse>;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        let opt = ctx.opt();
        let events =
            client.chain_get_events_by_txn_hash(opt.hash, Some(GetEventOption { decode: true }))?;
        Ok(events)
    }
}
