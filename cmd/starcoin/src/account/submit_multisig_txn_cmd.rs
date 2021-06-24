// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use starcoin_crypto::hash::HashValue;
use starcoin_vm_types::transaction::SignedUserTransaction;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "submit-multisig-txn")]
pub struct SubmitMultisigTxnOpt {
    #[structopt(name = "multisigned-txn", required = true, parse(from_os_str))]
    /// file contains the multi-signed txn
    signed_txn_file: PathBuf,

    #[structopt(
        short = "b",
        name = "blocking-mode",
        long = "blocking",
        help = "blocking wait txn mined"
    )]
    blocking: bool,
}

pub struct SubmitMultiSignedTxnCommand;

impl CommandAction for SubmitMultiSignedTxnCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = SubmitMultisigTxnOpt;
    type ReturnItem = HashValue;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let opt = ctx.opt();
        let client = ctx.state().client();
        let signed_txn: SignedUserTransaction =
            bcs_ext::from_bytes(&std::fs::read(opt.signed_txn_file.as_path())?)?;

        eprintln!(
            "Prepare to submit the transaction: \n {}",
            serde_json::to_string_pretty(&signed_txn)?
        );

        let txn_hash = signed_txn.id();
        client.submit_transaction(signed_txn)?;

        eprintln!("txn {:#x} submitted.", txn_hash);

        if opt.blocking {
            ctx.state().watch_txn(txn_hash)?;
        }
        Ok(txn_hash)
    }
}
