// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use itertools::Itertools;
use scmd::{CommandAction, ExecContext};
use serde::Deserialize;
use serde::Serialize;
use starcoin_crypto::ed25519::Ed25519PublicKey;
use starcoin_crypto::ValidCryptoMaterialStringExt;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::transaction::authenticator::AuthenticationKey;
use starcoin_vm_types::transaction::authenticator::AccountPublicKey;
use structopt::StructOpt;

/// Derive an address by public key.
#[derive(Debug, StructOpt)]
#[structopt(name = "derive-address")]
pub struct DeriveAddressOpt {
    #[structopt(short = "p", long = "pubkey", required=true, min_values=1, max_values=32, parse(try_from_str=Ed25519PublicKey::from_encoded_string))]
    /// public key used to derive address.If multi public keys is provided, a multi-sig account address is derived.
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
        anyhow::ensure!(
            !opt.public_key.is_empty(),
            "at least one public key is provided"
        );
        let account_key = if opt.public_key.len() == 1 {
            let public_key = opt
                .public_key
                .first()
                .cloned()
                .expect("must at least have one public key");
            AccountPublicKey::single(public_key)
        } else {
            let threshold = opt.threshold.unwrap_or(opt.public_key.len() as u8);
            // sort the public key to make account address derivation stable.
            let mut pubkeys = opt.public_key.clone();
            pubkeys.sort_by_key(|k| k.to_bytes());
            // remove repeat public keys
            let pubkeys = pubkeys
                .into_iter()
                .unique_by(|k| k.to_bytes())
                .collect::<Vec<_>>();
            AccountPublicKey::multi(pubkeys, threshold)?
        };
        let address = account_key.derived_address();
        let receipt_identifier = address.to_bech32();
        Ok(DerivedAddressData {
            address,
            auth_key: account_key.authentication_key(),
            receipt_identifier,
            public_key: account_key,
        })
    }
}

#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
pub struct DerivedAddressData {
    pub address: AccountAddress,
    pub auth_key: AuthenticationKey,
    pub receipt_identifier: String,
    pub public_key: AccountPublicKey,
}
