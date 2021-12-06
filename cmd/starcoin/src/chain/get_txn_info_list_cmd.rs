// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use starcoin_rpc_api::types::TransactionInfoView;
use structopt::StructOpt;

/// Get transaction infos list
#[derive(Debug, StructOpt)]
#[structopt(name = "get-txn-info-list", alias = "get_txn_info_list")]
pub struct GetTransactionInfoListOpt {
    /// The global transaction index for start scan.
    #[structopt(name = "start-index", alias = "start_index", long, short = "s")]
    start_index: u64,

    #[structopt(name = "reverse", long, short = "r")]
    reverse: Option<bool>,

    #[structopt(name = "count", long, short = "c", default_value = "32")]
    count: u64,
}

pub struct GetTransactionInfoListCommand;

impl CommandAction for GetTransactionInfoListCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = GetTransactionInfoListOpt;
    type ReturnItem = Vec<TransactionInfoView>;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        let opt = ctx.opt();
        let txn_infos = client.chain_get_transaction_infos(
            opt.start_index,
            opt.reverse.unwrap_or(false),
            opt.count,
        )?;
        Ok(txn_infos)
    }
}
