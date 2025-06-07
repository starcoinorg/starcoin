// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use clap::Parser;
use scmd::{CommandAction, ExecContext};
use serde::{Deserialize, Serialize};
use starcoin_vm2_types::{genesis_config::ChainId, sign_message::SignedMessage};

/// Verify the message signed by the sign command.
#[derive(Debug, Parser)]
#[clap(name = "verify-sign-message")]
pub struct VerifySignMessageOpt {
    #[clap(short = 'm')]
    signed_message: SignedMessage,
}

pub struct VerifySignMessageCmd;

impl CommandAction for VerifySignMessageCmd {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = VerifySignMessageOpt;
    type ReturnItem = VerifyResult;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let opt = ctx.opt();
        let state = ctx.state().vm2()?;
        let signed_message = opt.signed_message.clone();
        let account_resource = state.get_account_resource(signed_message.account)?;

        let result = signed_message.check_signature().and_then(|_| {
            signed_message.check_account(
                ChainId::new(state.net().chain_id().id()),
                Some(&account_resource),
            )
        });

        Ok(VerifyResult {
            ok: result.is_ok(),
            error: result.err().map(|e| e.to_string()),
            msg: signed_message,
        })
    }
}

#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
pub struct VerifyResult {
    pub ok: bool,
    pub error: Option<String>,
    pub msg: SignedMessage,
}
