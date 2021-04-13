use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use starcoin_rpc_api::types::EpochUncleSummaryView;
use structopt::StructOpt;

/// Show uncle summary in a epoch.
#[derive(Debug, StructOpt)]
#[structopt(name = "epoch_uncle_summary_by_number")]
pub struct EpochUncleSummaryByNumberOpt {
    #[structopt(name = "number", long, short = "n", default_value = "0")]
    number: u64,
}

pub struct EpochUncleSummaryByNumberCommand;

impl CommandAction for EpochUncleSummaryByNumberCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = EpochUncleSummaryByNumberOpt;
    type ReturnItem = EpochUncleSummaryView;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        ctx.state()
            .client()
            .epoch_uncle_summary_by_number(ctx.opt().number)
    }
}
