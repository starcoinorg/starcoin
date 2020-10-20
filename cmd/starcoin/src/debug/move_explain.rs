// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use errmapgen::ErrorContext;
use scmd::{CommandAction, ExecContext};
use starcoin_move_explain::get_explanation;
use starcoin_vm_types::account_address::AccountAddress;
use starcoin_vm_types::{identifier::Identifier, language_storage::ModuleId};
use structopt::StructOpt;

///Explain Move abort codes. Errors are defined as
///a global category + module-specific reason for the error.
#[derive(Debug, StructOpt)]
#[structopt(name = "move_explain")]
pub struct MoveExplainOpt {
    /// The location (module id) returned with a `MoveAbort` error
    #[structopt(short = "l")]
    location: String,
    /// The abort code returned with a `MoveAbort` error
    #[structopt(short = "a")]
    abort_code: u64,
}

pub struct MoveExplain;

impl CommandAction for MoveExplain {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = MoveExplainOpt;
    type ReturnItem = Option<ErrorContext>;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let opt = ctx.opt();
        let mut location = opt.location.trim().split("::");
        let mut address_literal = location.next().expect("Could not find address").to_string();
        let module_name = location
            .next()
            .expect("Could not find module name")
            .to_string();
        if !address_literal.starts_with("0x") {
            address_literal = format!("0x{}", address_literal);
        }
        let module_id = ModuleId::new(
            AccountAddress::from_hex_literal(&address_literal)
                .expect("Unable to parse module address"),
            Identifier::new(module_name).expect("Invalid module name encountered"),
        );

        Ok(get_explanation(&module_id, opt.abort_code))
    }
}
