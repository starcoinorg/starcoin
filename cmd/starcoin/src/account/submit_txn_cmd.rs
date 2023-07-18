// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use clap::Parser;

use scmd::{CommandAction, ExecContext};
use starcoin_vm_types::transaction::SignedUserTransaction;

use crate::cli_state::CliState;
use crate::view::{ExecutionOutputView, FilePathOrHex};
use crate::StarcoinOpt;

#[derive(Debug, Parser)]
/// Submit a SignedUserTransaction file or hex to transaction pool.
#[clap(name = "submit-txn", alias = "submit-multisig-txn")]
pub struct SubmitTxnOpt {
    #[clap(name = "signed-txn-file-or-hex", required = true)]
    /// file contains the signed txn or hex string
    signed_txn_file_or_hex: FilePathOrHex,

    #[clap(
        short = 'b',
        name = "blocking-mode",
        long = "blocking",
        help = "blocking wait tansaction(txn) mined"
    )]
    blocking: bool,
}

pub struct SubmitSignedTxnCommand;

impl CommandAction for SubmitSignedTxnCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = SubmitTxnOpt;
    type ReturnItem = ExecutionOutputView;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let opt = ctx.opt();

        let signed_txn: SignedUserTransaction =
            bcs_ext::from_bytes(opt.signed_txn_file_or_hex.as_bytes()?.as_slice())?;

        ctx.state().submit_txn(signed_txn, opt.blocking)
    }
}
