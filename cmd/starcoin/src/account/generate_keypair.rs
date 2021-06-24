// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::{bail, format_err, Result};
use scmd::{CommandAction, ExecContext};
use serde::Deserialize;
use serde::Serialize;
use starcoin_crypto::keygen::KeyGen;
use starcoin_crypto::ValidCryptoMaterialStringExt;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::transaction::authenticator::AuthenticationKey;
use starcoin_vm_types::transaction::authenticator::{AccountPrivateKey, AccountPublicKey};
use std::convert::TryInto;
use structopt::StructOpt;

/// Generate keypair
#[derive(Debug, StructOpt)]
#[structopt(name = "generate-keypair")]
pub struct GenerateKeypairOpt {
    #[structopt(short = "s", name = "seed")]
    /// random seed for generate keypair, should been a 32 bytes hex string.
    seed: Option<String>,
    /// How many keypair to generate
    #[structopt(short = "c", name = "count")]
    count: Option<u32>,
}

pub struct GenerateKeypairCommand;
const SEED_HEX_LENGTH: usize = 64;

impl CommandAction for GenerateKeypairCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = GenerateKeypairOpt;
    type ReturnItem = Vec<GenerateKeypairData>;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let opt = ctx.opt();
        let mut key_gen = if let Some(literal) = opt.seed.as_ref() {
            let literal = literal.strip_prefix("0x").unwrap_or(literal);
            if literal.len() != SEED_HEX_LENGTH {
                bail!(
                    "invalid seed argument, expect a {} len hex string, but got {}",
                    SEED_HEX_LENGTH,
                    literal.len()
                )
            }
            let seed = hex::decode(literal)?;
            let seed: [u8; 32] = seed
                .try_into()
                .map_err(|_| format_err!("invalid seed argument"))?;
            KeyGen::from_seed(seed)
        } else {
            KeyGen::from_os_rng()
        };
        let keypairs = (0..opt.count.unwrap_or(1))
            .into_iter()
            .map(|_| {
                let (private_key, public_key) = key_gen.generate_keypair();
                let account_public_key = AccountPublicKey::single(public_key);
                let account_private_key = AccountPrivateKey::Single(private_key);

                let address = account_public_key.derived_address();
                let receipt_identifier = address.to_bech32();
                GenerateKeypairData {
                    address,
                    auth_key: account_public_key.authentication_key(),
                    receipt_identifier,
                    public_key: account_public_key,
                    private_key: account_private_key
                        .to_encoded_string()
                        .expect("private key to string should success."),
                }
            })
            .collect::<Vec<_>>();

        Ok(keypairs)
    }
}

#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
pub struct GenerateKeypairData {
    pub address: AccountAddress,
    pub auth_key: AuthenticationKey,
    pub receipt_identifier: String,
    pub public_key: AccountPublicKey,
    pub private_key: String,
}
