// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use anyhow::{bail, Result};
use scmd::{CommandAction, ExecContext};
use serde::{Deserialize, Serialize};
use starcoin_config::{BaseConfig, StarcoinOpt};
use starcoin_genesis::Genesis;
use starcoin_logger::prelude::*;
use starcoin_types::genesis_config::{BuiltinNetwork, ChainNetwork, CustomNetwork};
use std::path::PathBuf;
use std::str::FromStr;
use structopt::StructOpt;

/// Generate starcoin genesis config in data_dir
#[derive(Debug, StructOpt)]
#[structopt(name = "genesis_config")]
pub struct GenGenesisConfigOpt {}

#[derive(Debug, Serialize, Deserialize)]
pub struct GenGenesisConfigResult {
    pub net: ChainNetwork,
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
        if global_opt.data_dir.is_none() {
            warn!("data_dir option is none, use default data_dir.")
        }
        let base = BaseConfig::new(
            global_opt.net.clone().unwrap_or_default(),
            global_opt.data_dir.clone(),
        );
        if !base.net().is_custom() {
            bail!("Only allow generate custom chain network config.");
        }
        if base
            .data_dir()
            .join(CustomNetwork::GENESIS_CONFIG_FILE_NAME)
            .exists()
        {
            bail!(
                "Genesis config file is exists in dir {:?}.",
                base.data_dir()
            );
        }
        if base.data_dir().join(Genesis::GENESIS_FILE_NAME).exists() {
            bail!("Genesis file is exists in dir {:?}.", base.data_dir());
        }
        let custom_network = base
            .net()
            .as_custom()
            .expect("This network must be custom chain network.");
        let genesis_config_name = custom_network.genesis_config_name();
        match BuiltinNetwork::from_str(genesis_config_name) {
            Ok(net) => {
                let genesis_config = net.genesis_config();
                let config_path = base
                    .data_dir()
                    .join(CustomNetwork::GENESIS_CONFIG_FILE_NAME);
                genesis_config.save(config_path.as_path())?;
                Ok(GenGenesisConfigResult {
                    net: ChainNetwork::new_custom(
                        custom_network.chain_name().to_string(),
                        custom_network.chain_id(),
                        None,
                    )?,
                    config_path,
                })
            }
            Err(_) => {
                bail!(
                    "Can not find builtin network by name {} for genesis config template",
                    genesis_config_name
                );
            }
        }
    }
}
