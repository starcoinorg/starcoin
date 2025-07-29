// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use clap::Parser;
use scmd::{CommandAction, ExecContext};
use serde_json::Value;
use starcoin_rpc_client::Params;

/// Call rpc api command
///  Some examples:
///  ``` shell
///  dev call-api node.info
///  dev call-api chain.get_block_by_number [0]
///  ```
#[derive(Debug, Parser)]
#[clap(name = "call-api")]
pub struct CallApiOpt {
    #[clap(name = "rpc-api-name")]
    /// api name to call, example: node.info
    rpc_api_name: String,

    #[clap(name = "api-params")]
    /// api params, should be a json array string
    params: Option<String>,
}

pub struct CallApiCommand;

impl CommandAction for CallApiCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = CallApiOpt;
    type ReturnItem = Value;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let opt = ctx.opt();

        let params = match opt.params.as_ref() {
            Some(param) => serde_json::from_str(param.as_str())?,
            None => Params::None,
        };

        let result = ctx
            .state()
            .vm2()?
            .client()
            .call_raw_api(opt.rpc_api_name.as_str(), params)?;
        Ok(result)
    }
}
