// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use anyhow::{format_err, Result};
use starcoin_rpc_client::StateRootOption;
use starcoin_state_api::StateReaderExt;
use starcoin_vm_types::on_chain_config::DaoConfig;

pub fn get_dao_config(cli_state: &CliState) -> Result<DaoConfig> {
    let client = cli_state.client();
    let chain_state_reader = client.state_reader(StateRootOption::Latest)?;
    chain_state_reader
        .get_on_chain_config::<DaoConfig>()?
        .ok_or_else(|| format_err!("DaoConfig not exist on chain."))
}
