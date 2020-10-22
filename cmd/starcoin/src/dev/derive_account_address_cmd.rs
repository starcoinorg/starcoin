// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use serde::Deserialize;
use serde::Serialize;
use starcoin_crypto::ed25519::Ed25519PublicKey;
use starcoin_crypto::multi_ed25519::MultiEd25519PublicKey;
use starcoin_crypto::ValidCryptoMaterialStringExt;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::transaction;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "derive-address")]
pub struct DeriveAddressOpt {
    #[structopt(short = "p", required=true, min_values=1, max_values=32, parse(try_from_str=Ed25519PublicKey::from_encoded_string))]
    /// public key used to derive address.If multi public keys is provided, a mutli-sig account address is derived.
    public_key: Vec<Ed25519PublicKey>,

    #[structopt(short = "t", name = "threshold")]
    /// In multi-sig case, a threshold is needed, default to the num of public keys.
    threshold: Option<u8>,
}

pub struct DeriveAddressCommand;

impl CommandAction for DeriveAddressCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = DeriveAddressOpt;
    type ReturnItem = DerivedAddressData;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let opt = ctx.opt();
        let _client = ctx.state().client();
        anyhow::ensure!(
            !opt.public_key.is_empty(),
            "at least one public key is provided"
        );
        let auth_key = if opt.public_key.len() == 1 {
            transaction::authenticator::AuthenticationKey::ed25519(opt.public_key.first().unwrap())
        } else {
            let threshold = opt.threshold.unwrap_or(opt.public_key.len() as u8);

            let multi_public_key = {
                // sort the public key to make account address derivation stable.
                let mut pubkeys = opt.public_key.clone();
                pubkeys.sort_by_key(|k| k.to_bytes());

                MultiEd25519PublicKey::new(pubkeys, threshold)?
            };
            transaction::authenticator::AuthenticationKey::multi_ed25519(&multi_public_key)
        };

        Ok(DerivedAddressData {
            address: auth_key.derived_address(),
            auth_key_prefix: hex::encode(auth_key.prefix().to_vec()),
            auth_key: hex::encode(auth_key.to_vec()),
        })
    }
}

#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
pub struct DerivedAddressData {
    pub address: AccountAddress,
    pub auth_key: String,
    /// hex encoded
    pub auth_key_prefix: String,
}
