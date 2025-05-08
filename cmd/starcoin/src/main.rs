// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
use anyhow::Result;
use scmd::error::CmdError;
use scmd::CmdContext;
use starcoin_account_provider::ProviderFactory;
use starcoin_cmd::*;
use starcoin_cmd::{CliState, StarcoinOpt};
use starcoin_config::{Connect, G_APP_VERSION, G_CRATE_VERSION};
use starcoin_logger::prelude::*;
use starcoin_node_api::errors::NodeStartError;
use starcoin_rpc_client::RpcClient;
use std::sync::Arc;
use std::time::Duration;

/// This exit code means is that the node failed to start and required human intervention.
/// Node start script can do auto task when meet this exist code.
static G_EXIT_CODE_NEED_HELP: i32 = 120;

use run::run;

fn main() {
    match run::<CliState, StarcoinOpt>() {
        Ok(()) => {}
        Err(e) => {
            match e.downcast::<NodeStartError>() {
                Ok(e) => match e {
                    //TODO not suggest clean data dir in main network.
                    NodeStartError::LoadConfigError(e) => {
                        error!("{:?}, please fix config.", e);
                        std::process::exit(G_EXIT_CODE_NEED_HELP);
                    }
                    NodeStartError::StorageInitError(e) => {
                        error!("{:?}, please clean your data dir.", e);
                        std::process::exit(G_EXIT_CODE_NEED_HELP);
                    }
                    NodeStartError::GenesisError(e) => {
                        error!("{:?}, please clean your data dir.", e);
                        std::process::exit(G_EXIT_CODE_NEED_HELP);
                    }
                    NodeStartError::Other(e) => {
                        error!("Node exit for an unexpected error: {:?}", e);
                        std::process::exit(1);
                    }
                },
                Err(e) => match e.downcast::<CmdError>() {
                    Ok(e) => match e {
                        CmdError::ClapError(e) => {
                            println!("{}", e);
                        }
                        CmdError::Other(e) => {
                            error!("Starcoin cmd return error: {:?}", e);
                            std::process::exit(1);
                        }
                    },
                    Err(e) => {
                        error!("Starcoin cmd exits abnormally: {:?}", e);
                        std::process::exit(1);
                    }
                },
            }
        }
    }
}
