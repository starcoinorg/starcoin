// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::mutlisig_transaction::MultisigTransaction;
use crate::StarcoinOpt;
use anyhow::ensure;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use starcoin_crypto::hash::PlainCryptoHash;
use starcoin_types::transaction;
use starcoin_vm_types::account_address::{parse_address, AccountAddress};
use std::env::current_dir;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "partial-sign-txn")]
/// Partial sign a multisig transaction, output the partial-signed transaction to file.
pub struct PartialSignTxnOpt {
    #[structopt(short = "i", long, parse(from_os_str))]
    /// txn input file
    input: PathBuf,

    #[structopt(
        short = "o",
        long = "output-dir",
        name = "output-dir",
        parse(from_os_str)
    )]
    /// txn output dir
    output_dir: Option<PathBuf>,

    #[structopt(short = "s", parse(try_from_str = parse_address))]
    /// if empty, use default account
    signer: Option<AccountAddress>,
}

pub struct PartialSignTxnCommand;

impl CommandAction for PartialSignTxnCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = PartialSignTxnOpt;
    type ReturnItem = PathBuf;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        //let client = ctx.state().client();
        let opt = ctx.opt();
        let mut txn: MultisigTransaction = {
            let mut f = File::open(&opt.input)?;
            let mut data = vec![];
            f.read_to_end(&mut data)?;
            scs::from_bytes(data.as_slice())?
        };
        let wallet_account = ctx.state().get_account_or_default(opt.signer)?;
        let signer_address = wallet_account.address;
        ensure!(
            txn.can_signed_by(&wallet_account.public_key),
            "account {} cannot sign the txn",
            signer_address
        );

        // wallet sign txn should only return public key, and signature.
        // let caller do the assemble.
        let signed_txn = ctx
            .state()
            .client()
            .account_sign_multisig_txn(txn.raw_txn().clone(), signer_address)?;
        let (signer_public_key, signer_signature) = match signed_txn.authenticator() {
            transaction::authenticator::TransactionAuthenticator::Ed25519 {
                public_key,
                signature,
            } => (public_key, signature),
            transaction::authenticator::TransactionAuthenticator::MultiEd25519 { .. } => {
                unreachable!()
            }
        };
        txn.collect_signature(signer_public_key, signer_signature);

        let output_file_path = {
            let mut output_dir = opt.output_dir.clone().unwrap_or(current_dir()?);
            // use txn id's short str and signer as the file name
            let file_name = txn.raw_txn().crypto_hash().short_str();
            output_dir.push(file_name.as_str());
            output_dir.set_extension("multisig-txn.partial");
            output_dir
        };
        let mut file = File::create(output_file_path.clone())?;
        scs::serialize_into(&mut file, &txn)?;
        Ok(output_file_path)
    }
}
