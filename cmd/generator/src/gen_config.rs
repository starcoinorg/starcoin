// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::init_or_load_data_dir;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use serde::{Deserialize, Serialize};
use starcoin_account_api::AccountInfo;
use starcoin_config::StarcoinOpt;
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::*;
use starcoin_types::chain_config::ChainNetwork;
use std::path::PathBuf;
use structopt::StructOpt;

/// Generate starcoin config, account and genesis in data_dir
#[derive(Debug, StructOpt)]
#[structopt(name = "config")]
pub struct GenConfigOpt {
    ///Default account password, default is empty string.
    #[structopt(long, short = "s")]
    password: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GenConfigResult {
    pub net: ChainNetwork,
    pub config_path: PathBuf,
    pub account_info: AccountInfo,
    pub genesis: HashValue,
}

pub struct GenConfigCommand;

impl CommandAction for GenConfigCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = GenConfigOpt;
    type ReturnItem = GenConfigResult;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let opt = ctx.opt();
        let global_opt = ctx.global_opt();
        if global_opt.data_dir.is_none() {
            warn!("data_dir option is none, use default data_dir.")
        }
        let (config, .., genesis_hash, account) =
            init_or_load_data_dir(&global_opt, opt.password.clone())?;
        Ok(GenConfigResult {
            net: config.net(),
            config_path: config.data_dir().join(starcoin_config::CONFIG_FILE_PATH),
            account_info: account,
            genesis: genesis_hash,
        })
    }
}
