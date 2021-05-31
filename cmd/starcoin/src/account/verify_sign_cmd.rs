// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::view::BoolView;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use starcoin_account_api::AccountSignature;
use starcoin_crypto::ValidCryptoMaterialStringExt;
use starcoin_types::sign_message::SigningMessage;
use structopt::StructOpt;

/// Verify the the message signed by the sign command.
#[derive(Debug, StructOpt)]
#[structopt(name = "verify-sign-message")]
pub struct VerifySignMessageOpt {
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
    type ReturnItem = BoolView;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let opt = ctx.opt();
        let signed = AccountSignature::from_encoded_string(opt.signed.as_str())?;
        let result = signed.verify(&opt.source);
        Ok(BoolView {
            result: result.is_ok(),
        })
    }
}
