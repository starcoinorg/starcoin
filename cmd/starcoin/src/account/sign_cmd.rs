// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use starcoin_rpc_api::types::StrView;
use starcoin_types::sign_message::SigningMessage;
use starcoin_vm_types::account_address::AccountAddress;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "sign-message")]
pub struct SignMessageOpt {
    #[structopt(short = "s")]
    /// if `sender` is absent, use default account.
    sender: Option<AccountAddress>,

    #[structopt(short = "m", long = "message", name = "signing-message")]
    message: SigningMessage,
}

pub struct SignMessageCmd;

impl CommandAction for SignMessageCmd {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = SignMessageOpt;
    type ReturnItem = StrView<Vec<u8>>;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let opt = ctx.opt();
        let client = ctx.state().client();
        let sender = ctx.state().get_account_or_default(opt.sender)?;
        let signed_message = client.account_sign_message(sender.address, opt.message.clone())?;
        Ok(signed_message)
    }
}
