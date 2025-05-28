// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::str::FromStr;

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use clap::Parser;
use scmd::{CommandAction, ExecContext};
use starcoin_crypto::HashValue;
use starcoin_dag::consensusdb::consensus_state::ReachabilityView;

/// Get block info by number
#[derive(Debug, Parser, Clone)]
#[clap(name = "is-ancestor-of", alias = "is_ancestor_of")]
pub struct IsAncestorOfOpt {
    #[clap(name = "ancestor", long, short = 'a')]
    ancestor: String,

    #[clap(name = "descendants", long, short = 'd')]
    descendants: Vec<String>,
}

pub struct IsAncestorOfCommand;

impl CommandAction for IsAncestorOfCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = IsAncestorOfOpt;
    type ReturnItem = ReachabilityView;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let opt = ctx.opt().clone();
        ctx.state().client().is_ancestor_of(
            HashValue::from_str(&opt.ancestor)?,
            opt.descendants
                .into_iter()
                .map(|id| HashValue::from_str(&id).map_err(|e| anyhow::anyhow!("{:?}", e)))
                .collect::<Result<Vec<_>>>()?,
        )
    }
}
