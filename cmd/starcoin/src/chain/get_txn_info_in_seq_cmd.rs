use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use clap::Parser;
use scmd::{CommandAction, ExecContext};
use starcoin_crypto::HashValue;
use starcoin_rpc_api::types::TransactionInfoViewEnum;

#[derive(Debug, Parser)]
#[clap(name = "get-txn-info-in-seq", alias = "get_txn_info_in_seq")]
pub struct GetTransactionInfoInSeqOpt {
    #[clap(name = "block-hash")]
    block_hash: HashValue,
}

pub struct GetTransactionInfoInSeqCommand;

impl CommandAction for GetTransactionInfoInSeqCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = GetTransactionInfoInSeqOpt;
    type ReturnItem = Vec<TransactionInfoViewEnum>;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        let opt = ctx.opt();
        let result = client.chain_get_block_txn_infos_in_seq(opt.block_hash)?;
        Ok(result)
    }
}
