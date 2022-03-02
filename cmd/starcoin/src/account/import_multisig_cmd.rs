// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use itertools::Itertools;
use scmd::{CommandAction, ExecContext};
use starcoin_account_api::{AccountInfo, AccountPrivateKey};
use starcoin_crypto::ed25519::{Ed25519PrivateKey, Ed25519PublicKey};
use starcoin_crypto::multi_ed25519::multi_shard::MultiEd25519KeyShard;
use starcoin_crypto::{PrivateKey, ValidCryptoMaterial, ValidCryptoMaterialStringExt};
use starcoin_vm_types::account_address::AccountAddress;
use std::path::PathBuf;
use structopt::StructOpt;

/// Import multisin account.
#[derive(Debug, StructOpt)]
#[structopt(name = "import-multisig")]
pub struct ImportMultisigOpt {
    #[structopt(short = "p", default_value = "")]
    /// a password to protect imported account.
    password: String,

    #[structopt(name = "addr", long)]
    /// if account_address is absent, generate address by public_key.
    account_address: Option<AccountAddress>,

    #[structopt(long = "pubkey", max_values = 32, parse(try_from_str = Ed25519PublicKey::from_encoded_string))]
    /// public keys of other participants in this multisig account.
    public_keys: Vec<Ed25519PublicKey>,

    #[structopt(short = "t", name = "threshold")]
    /// In multi-sig case, a threshold is needed.
    threshold: u8,

    #[structopt(long = "prikey", max_values = 32, parse(try_from_str = Ed25519PrivateKey::from_encoded_string))]
    /// hex encoded private key, if you control multi private keys, provide multi args.
    private_keys: Vec<Ed25519PrivateKey>,

    #[structopt(long = "prikey-file", max_values = 32)]
    /// private key file contain the hex-encoded private key, if you control multi private keys, provide multi args.
    private_key_files: Vec<PathBuf>,
}

pub struct ImportMultisigCommand;

impl CommandAction for ImportMultisigCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = ImportMultisigOpt;
    type ReturnItem = AccountInfo;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().account_client();
        let opt: &ImportMultisigOpt = ctx.opt();

        let mut private_keys = vec![];
        for p in &opt.private_keys {
            private_keys.push(p.clone());
        }
        for file in &opt.private_key_files {
            let prikey =
                Ed25519PrivateKey::from_encoded_string(std::fs::read_to_string(file)?.trim())?;
            private_keys.push(prikey);
        }
        if private_keys.is_empty() {
            anyhow::bail!("require at least one private key or private file");
        }

        let public_keys = {
            let mut keys = opt.public_keys.to_vec();
            keys.extend(private_keys.iter().map(|p| p.public_key()));
            // sort all public keys by its bytes to make sure same public key set always generate same auth key.
            keys.sort_by_key(|k| k.to_bytes());
            // remove repeat public keys, if use add repeat public_key or private key.
            keys.into_iter()
                .unique_by(|k| k.to_bytes())
                .collect::<Vec<_>>()
        };
        let threshold = opt.threshold;
        let private_key = AccountPrivateKey::Multi(MultiEd25519KeyShard::new_multi(
            public_keys,
            threshold,
            private_keys,
        )?);

        let address = opt
            .account_address
            .unwrap_or_else(|| private_key.public_key().derived_address());
        let account = client.import_account(
            address,
            private_key.to_bytes().to_vec(),
            opt.password.clone(),
        )?;
        Ok(account)
    }

    fn skip_history(&self, _ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>) -> bool {
        true
    }
}
