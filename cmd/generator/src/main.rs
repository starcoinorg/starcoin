// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use scmd::CmdContext;
use starcoin_config::StarcoinOpt;
use starcoin_generator::cli_state::CliState;
use starcoin_generator::gen_config::GenConfigCommand;
use starcoin_generator::gen_data::GenDataCommand;
use starcoin_logger::prelude::*;

fn run() -> Result<()> {
    let context = CmdContext::<CliState, StarcoinOpt>::with_state(CliState);
    context
        .command(GenConfigCommand)
        .command(GenDataCommand)
        .exec()
}

/// A tools for generate starcoin config and data.
fn main() {
    let _logger_handle = starcoin_logger::init();
    if let Err(e) = run() {
        error!("error: {:?}", e);
    }
}
