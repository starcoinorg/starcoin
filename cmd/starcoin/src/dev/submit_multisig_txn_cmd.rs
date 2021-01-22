// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::mutlisig_transaction::MultisigTransaction;
use crate::StarcoinOpt;
use anyhow::{bail, ensure, Result};
use scmd::{CommandAction, ExecContext};
use starcoin_crypto::hash::HashValue;
use starcoin_vm_types::transaction::SignedUserTransaction;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "submit-multisig-txn")]
pub struct SubmitMultisigTxnOpt {
    #[structopt(name = "partial-signed-txn", required = true, parse(from_os_str))]
    /// partial signed txn
    partial_signed_txns: Vec<PathBuf>,
    #[structopt(
        short = "b",
        name = "blocking-mode",
        long = "blocking",
        help = "blocking wait txn mined"
    )]
    blocking: bool,
}

pub struct ExecuteMultiSignedTxnCommand;

impl CommandAction for ExecuteMultiSignedTxnCommand {
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
        let signed_txn = assemble_multisig_txn(opt.partial_signed_txns.clone())?;
        let txn_hash = signed_txn.id();
        client.submit_transaction(signed_txn)?;

        println!("txn {:#x} submitted.", txn_hash);

        if opt.blocking {
            ctx.state().watch_txn(txn_hash)?;
        }
        Ok(txn_hash)
    }
}
fn assemble_multisig_txn(partial: Vec<PathBuf>) -> Result<SignedUserTransaction> {
    anyhow::ensure!(
        !partial.is_empty(),
        "multisig txn should contain at least 1 signers"
    );

    let mut txns = vec![];
    for p in &partial {
        let mut f = File::open(p)?;
        let mut data = vec![];
        f.read_to_end(&mut data)?;
        let txn: MultisigTransaction = bcs_ext::from_bytes(data.as_slice())?;
        txns.push(txn);
    }
    let mut first_txn = txns.swap_remove(0);

    for txn in &txns {
        // ensure we are in the same channel
        ensure!(txn.raw_txn() == first_txn.raw_txn(), "raw txn mismatch");
        ensure!(
            txn.multi_public_key() == first_txn.multi_public_key(),
            "multisig account mismatch"
        );

        for (k, s) in txn.collected_signatures() {
            if !first_txn.collect_signature(k.clone(), s.clone()) {
                bail!(
                    "signer of public key {:?} is not part of the mutlisig account",
                    k
                );
            }
        }
    }

    let multi_signed_txn = first_txn.into_signed_txn()?;
    Ok(multi_signed_txn)
}
