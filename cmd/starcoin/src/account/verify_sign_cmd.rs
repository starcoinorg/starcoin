// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::view::StringView;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use starcoin_crypto::ed25519::{Ed25519PublicKey, Ed25519Signature};
use starcoin_crypto::{ValidCryptoMaterialStringExt, VerifyingKey};
use starcoin_types::sign_message::SigningMessage;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "verify-sign-message")]
pub struct VerifySignMessageOpt {
    #[structopt(short = "k")]
    /// if `to` account not exist on chain, must provide public_key of the account.
    public_key: String,

    #[structopt(short = "m", long = "source", name = "source-message")]
    source: SigningMessage,

    #[structopt(short = "d", long = "signed", name = "signed-message")]
    signed: String,
}

pub struct VerifySignMessageCmd;

impl CommandAction for VerifySignMessageCmd {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = VerifySignMessageOpt;
    type ReturnItem = StringView;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let opt = ctx.opt();
        let public_key = Ed25519PublicKey::from_encoded_string(opt.public_key.as_str())?;
        let signature = Ed25519Signature::from_encoded_string(opt.signed.as_str())?;
        public_key.verify_struct_signature(&opt.source, &signature)?;
        Ok(StringView {
            result: "ok".parse()?,
        })
    }
}
