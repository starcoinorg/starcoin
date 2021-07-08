// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::view::BoolView;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use starcoin_types::sign_message::SignedMessage;
use structopt::StructOpt;

/// Verify the the message signed by the sign command.
#[derive(Debug, StructOpt)]
#[structopt(name = "verify-sign-message")]
pub struct VerifySignMessageOpt {
    #[structopt(short = "m")]
    signed_message: SignedMessage,
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
        let state = ctx.state();
        let signed_message = opt.signed_message.clone();
        let account_resource = state.get_account_resource(signed_message.account)?;

        let result = signed_message
            .check_signature()
            .and_then(|_| signed_message.check_account(account_resource.as_ref()));
        if let Err(e) = result.as_ref() {
            eprintln!("check signed message error: {}", e)
        }
        Ok(BoolView {
            result: result.is_ok(),
        })
    }
}
