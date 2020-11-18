use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use starcoin_crypto::HashValue;
use starcoin_rpc_api::types::BlockHeaderView;
use structopt::StructOpt;

/// Show the path from uncle parent block to mint block.
#[derive(Debug, StructOpt)]
#[structopt(name = "path")]
pub struct UnclePathOpt {
    #[structopt(short = "b")]
    block_id: HashValue,
    #[structopt(short = "u")]
    uncle_id: HashValue,
}

pub struct UnclePathCommand;

impl CommandAction for UnclePathCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = UnclePathOpt;
    type ReturnItem = Vec<BlockHeaderView>;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        let block_headers = client.uncle_path(ctx.opt().block_id, ctx.opt().uncle_id)?;
        Ok(block_headers)
    }
}
