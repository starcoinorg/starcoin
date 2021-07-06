// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::{format_err, Result};
use scmd::{CommandAction, ExecContext};
use starcoin_vm_types::account_address::AccountAddress;
use starcoin_vm_types::vm_status::AbortLocation;
use starcoin_vm_types::{identifier::Identifier, language_storage::ModuleId};
use structopt::StructOpt;
use vm_status_translator::{explain_move_abort, MoveAbortExplain};

///Explain Move abort codes. Errors are defined as
///a global category + module-specific reason for the error.
#[derive(Debug, StructOpt)]
#[structopt(name = "move-explain", alias = "move_explain")]
pub struct MoveExplainOpt {
    /// The location (module id) returned with a `MoveAbort` error
    #[structopt(short = "l")]
    location: Option<String>,
    /// The abort code returned with a `MoveAbort` error
    #[structopt(short = "a")]
    abort_code: u64,
}

pub struct MoveExplain;

impl CommandAction for MoveExplain {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = MoveExplainOpt;
    type ReturnItem = MoveAbortExplain;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let opt = ctx.opt();
        match opt.location {
            Some(_) => {
                let mut location = opt.location.as_ref().unwrap().trim().split("::");
                let mut address_literal = location
                    .next()
                    .ok_or_else(|| format_err!("Could not find address"))?
                    .to_string();
                let module_name = location
                    .next()
                    .ok_or_else(|| format_err!("Could not find module name"))?
                    .to_string();
                if !address_literal.starts_with("0x") {
                    address_literal = format!("0x{}", address_literal);
                }
                let module_id = ModuleId::new(
                    AccountAddress::from_hex_literal(&address_literal)?,
                    Identifier::new(module_name)?,
                );

                let explain = explain_move_abort(AbortLocation::Module(module_id), opt.abort_code);

                Ok(explain)
            }
            None => Ok(explain_move_abort(AbortLocation::Script, opt.abort_code)),
        }
    }
}
