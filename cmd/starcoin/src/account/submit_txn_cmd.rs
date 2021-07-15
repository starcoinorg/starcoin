// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::view::{ExecutionOutputView, FilePathOrHex};
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use starcoin_rpc_api::types::SignedUserTransactionView;
use starcoin_vm_types::transaction::SignedUserTransaction;
use std::convert::TryInto;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
/// Submit a SignedTransaction file or hex to transaction pool.
#[structopt(name = "submit-txn", alias = "submit-multisig-txn")]
pub struct SubmitTxnOpt {
    #[structopt(name = "signed-txn-file-or-hex", required = true)]
    /// file contains the signed txn or hex string
    signed_txn_file_or_hex: FilePathOrHex,

    #[structopt(
        short = "b",
        name = "blocking-mode",
        long = "blocking",
        help = "blocking wait txn mined"
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
        let client = ctx.state().client();
        let signed_txn: SignedUserTransaction =
            bcs_ext::from_bytes(opt.signed_txn_file_or_hex.as_bytes()?.as_slice())?;

        let mut signed_txn_view: SignedUserTransactionView = signed_txn.clone().try_into()?;
        signed_txn_view.raw_txn.decoded_payload =
            Some(ctx.state().decode_txn_payload(signed_txn.payload())?.into());

        eprintln!(
            "Prepare to submit the transaction: \n {}",
            serde_json::to_string_pretty(&signed_txn_view)?
        );
        let txn_hash = signed_txn.id();
        client.submit_transaction(signed_txn)?;

        eprintln!("txn {:#x} submitted.", txn_hash);

        if opt.blocking {
            ctx.state().watch_txn(txn_hash)
        } else {
            Ok(ExecutionOutputView::new(txn_hash))
        }
    }
}
