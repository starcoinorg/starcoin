// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use serde::{Deserialize, Serialize};
use starcoin_rpc_api::types::FactoryAction;
use structopt::StructOpt;

///Get and set txn factory status.
#[derive(Debug, StructOpt)]
#[structopt(name = "txfactory_status")]
pub struct TxFactoryOpt {
    #[structopt(name = "action")]
    action: FactoryAction,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TxFactoryStatus {
    pub is_start: bool,
}

impl TxFactoryStatus {
    fn new(is_start: bool) -> Self {
        TxFactoryStatus { is_start }
    }
}

pub struct TxFactoryCommand;

impl CommandAction for TxFactoryCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = TxFactoryOpt;
    type ReturnItem = TxFactoryStatus;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let opt = ctx.opt();
        let client = ctx.state().client();
        let result = client.debug_txfactory_status(opt.action.clone())?;
        Ok(TxFactoryStatus::new(result))
    }
}
