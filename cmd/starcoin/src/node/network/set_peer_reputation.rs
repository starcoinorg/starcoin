// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "set_reputation")]
pub struct ReportPeerOpt {
    #[structopt(name = "peer")]
    /// format: multiaddr/p2p/peer_id
    peer: String,
    #[structopt(subcommand)]
    /// set reputation
    reputation: Reputation,
}

#[derive(Debug, StructOpt)]
enum Reputation {
    /// banned the peer
    Banned,
    /// set the reput change score for the peer
    Reput {
        #[structopt(long)]
        score: i32,
    },
}

pub struct SetPeerReputation;

impl CommandAction for SetPeerReputation {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = ReportPeerOpt;
    type ReturnItem = ();

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let opt = ctx.opt();
        let client = ctx.state().client();
        let reput = match opt.reputation {
            Reputation::Banned => i32::MIN,
            Reputation::Reput { score } => score,
        };
        client.set_peer_reputation(ctx.opt().peer.clone(), reput)
    }
}
