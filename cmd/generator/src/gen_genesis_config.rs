// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use anyhow::{bail, ensure, Result};
use scmd::{CommandAction, ExecContext};
use serde::{Deserialize, Serialize};
use starcoin_config::ChainNetworkID;
use starcoin_config::{BaseConfig, StarcoinOpt};
use starcoin_genesis::Genesis;
use starcoin_logger::prelude::*;
use std::path::PathBuf;
use structopt::StructOpt;

/// Generate starcoin genesis config in data_dir
#[derive(Debug, StructOpt)]
#[structopt(name = "genesis_config")]
pub struct GenGenesisConfigOpt {}

#[derive(Debug, Serialize, Deserialize)]
pub struct GenGenesisConfigResult {
    pub net: ChainNetworkID,
    pub config_path: PathBuf,
}

pub struct GenGenesisConfigCommand;

impl CommandAction for GenGenesisConfigCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = GenGenesisConfigOpt;
    type ReturnItem = GenGenesisConfigResult;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let global_opt = ctx.global_opt();
        if global_opt.base_data_dir.is_none() {
            warn!("data_dir option is none, use default data_dir.")
        }
        ensure!(
            global_opt.genesis_config.is_some(),
            "please set genesis-config option"
        );
        let base = BaseConfig::load_with_opt(global_opt)?;
        if !base.net().is_custom() {
            bail!("Only allow generate custom chain network config.");
        }
        if base.data_dir().join(Genesis::GENESIS_FILE_NAME).exists() {
            bail!("Genesis file is exists in dir {:?}.", base.data_dir());
        }

        let config_path = base
            .data_dir()
            .join(starcoin_config::GENESIS_CONFIG_FILE_NAME);
        // genesis config file auto generate in BaseConfig::default_with_opt
        ensure!(
            config_path.exists(),
            "Genesis Config should exist in {:?}",
            config_path
        );
        Ok(GenGenesisConfigResult {
            net: base.net().id().clone(),
            config_path,
        })
    }
}
